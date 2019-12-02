use futures::task::{Context, Poll};
use futures::{Stream, StreamExt};

use crate::Id;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;


pub trait Service : Stream<Item = <Self as Service>::Data> {
    type Data: 'static + Send;

    fn start(&mut self, mailbox: ServiceMailbox);
    fn stop(self);
}

pub struct ServiceSubscription<T: 'static> {
    inner: Box<dyn ServiceMap<T>>,
}

impl<T: 'static + Send> ServiceSubscription<T> {
    pub fn new<S, Mapper>(service: S, mapper: Mapper) -> Self
    where
        S: Unpin + Service + Send + 'static,
        Mapper: 'static + Fn(S::Data) -> T + Send
    {
        let ret = ServiceMapDirect {
            service,
            mapper,
            id: Id::new(),
            phantom_data: PhantomData,
            phantom_t: PhantomData,
        };
        ServiceSubscription {
            inner: Box::new(ret),
        }
    }

    pub fn start(&mut self, mailbox: ServiceMailbox) {
        self.inner.start(mailbox);
    }

    pub fn map<U: 'static, Mapper: 'static + Fn(T) -> U + Send + Sync>(
        self,
        mapper: Arc<Mapper>,
    ) -> ServiceSubscription<U> {
        let ret = ServiceMapped {
            inner: self.inner,
            mapper,
        };
        ServiceSubscription {
            inner: Box::new(ret),
        }
    }

    pub fn stop(self) {
        self.inner.stop();
    }

    pub fn id(&self) -> Id {
        self.inner.id()
    }
}

impl<T: 'static> Stream for ServiceSubscription<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

trait ServiceMap<T> : Send {
    fn start(&mut self, mailbox: ServiceMailbox);
    fn stop(self: Box<Self>);
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<T>>;
    fn size_hint(&self) -> (usize, Option<usize>);
    fn id(&self) -> Id;
}

struct ServiceMapDirect<
    T: Send,
    Mapper: Fn(S::Data) -> T + Send,
    S: Service + Unpin + Send,
> {
    service: S,
    mapper: Mapper,
    id: Id,
    phantom_data: std::marker::PhantomData<S::Data>,
    phantom_t: std::marker::PhantomData<T>,
}

impl<T, Mapper, S> ServiceMap<T>
    for ServiceMapDirect<T, Mapper, S>
where
    T: Send,
    S: Service + Unpin + Send,
    Mapper: Fn(S::Data) -> T + Send,
{
    fn start(&mut self, mailbox: ServiceMailbox) {
        self.service.start(mailbox);
    }

    fn stop(self: Box<Self>) {
        self.service.stop();
    }

    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<T>> {
        let ret = self.service.poll_next_unpin(cx);
        match ret {
            Poll::Ready(Some(data)) => Poll::Ready(Some((self.mapper)(data))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.service.size_hint()
    }

    fn id(&self) -> Id {
        self.id
    }
}

struct ServiceMapped<T, U, Mapper: Fn(T) -> U> {
    inner: Box<dyn ServiceMap<T>>,
    mapper: Arc<Mapper>,
}

impl<T, U, Mapper: Fn(T) -> U + Send + Sync> ServiceMap<U>
    for ServiceMapped<T, U, Mapper>
{
    fn start(&mut self, mailbox: ServiceMailbox) {
        self.inner.start(mailbox);
    }

    fn stop(self: Box<Self>) {
        self.inner.stop();
    }

    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<U>> {
        let ret = self.inner.poll_next(cx);
        match ret {
            Poll::Ready(Some(data)) => Poll::Ready(Some((self.mapper)(data))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    fn id(&self) -> Id {
        self.inner.id()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TxServiceMessage {
    Frontend(Vec<u8>),
    RunJs(String),
    LoadCss(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RxServiceMessage {
    Frontend(String),
}

pub struct ServiceMailbox {
    pub(crate) rx: UnboundedReceiver<RxServiceMessage>,
    pub(crate) tx: UnboundedSender<TxServiceMessage>,
}

impl ServiceMailbox {
    pub fn run_js<T: Into<String>>(&self, code: T) {
        let _ = self.tx.unbounded_send(TxServiceMessage::RunJs(code.into()));
    }

    pub fn send_data(&self, data: Vec<u8>) {
        let _ = self.tx.unbounded_send(TxServiceMessage::Frontend(data));
    }

    pub fn load_css<T: Into<String>>(&self, css: T) {
        let _ = self.tx.unbounded_send(TxServiceMessage::LoadCss(css.into()));
    }
}

impl Stream for ServiceMailbox {
    type Item = RxServiceMessage;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.rx).poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.rx.size_hint()
    }
}

pub trait SimpleService {
    type Data: 'static + Send;

    fn run(self, mailbox: ServiceMailbox, sender: UnboundedSender<Self::Data>);
}

pub struct SimpleServiceContainer<S: SimpleService> {
    service: Option<S>,
    tx: UnboundedSender<S::Data>,
    rx: UnboundedReceiver<S::Data>,
}

impl<S: SimpleService + Unpin> SimpleServiceContainer<S> {
    pub fn new(service: S) -> Self {
        let (tx,rx) = unbounded();
        Self {
            service: Some(service),
            tx,
            rx
        }
    }

}

impl<S: SimpleService + Unpin> Service for SimpleServiceContainer<S> {
    type Data = S::Data;

    fn start(&mut self, mailbox: ServiceMailbox) {
        let service = self.service.take().unwrap();
        let tx = self.tx.clone();
        service.run(mailbox, tx);
    }

    fn stop(self) {
    }
}

impl<S: SimpleService + Unpin> Stream for SimpleServiceContainer<S> {
    type Item = S::Data;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.rx).poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.rx.size_hint()
    }
}