use futures::task::{Context, Poll};
use futures::{Stream, StreamExt};

use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use crate::Id;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use serde::{Serialize, Deserialize};

pub trait ServiceProcess<T>: Stream<Item = T> {
    fn stop(self);
    fn setup(&mut self, _mailbox: ServiceMailbox) {}
}

pub trait Service {
    type Data: 'static + Send;
    type Output: ServiceProcess<Self::Data> + Unpin + 'static + Send;

    fn start(self) -> Self::Output;
}

pub struct ServiceSubscription<T: 'static> {
    inner: Box<dyn ServiceProcessMap<T>>,
}

impl<T: 'static + Send> ServiceSubscription<T> {
    pub fn start<S: Service, Mapper: 'static + Fn(S::Data) -> T + Send>(
        service: S,
        mapper: Mapper,
    ) -> Self {
        let process = service.start();
        let ret = ServiceProcessMapDirect {
            process,
            mapper,
            id: Id::new(),
            phantom_data: PhantomData,
            phantom_t: PhantomData,
        };
        ServiceSubscription {
            inner: Box::new(ret),
        }
    }

    pub fn map<U: 'static, Mapper: 'static + Fn(T) -> U + Send + Sync>(
        self,
        mapper: Arc<Mapper>,
    ) -> ServiceSubscription<U> {
        let ret = ServiceProcessMapped {
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

    pub fn setup(&mut self, mailbox: ServiceMailbox) {
        self.inner.setup(mailbox)
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

trait ServiceProcessMap<T> : Send {
    fn stop(self: Box<Self>);
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<T>>;
    fn size_hint(&self) -> (usize, Option<usize>);
    fn id(&self) -> Id;
    fn setup(&mut self, mailbox: ServiceMailbox);
}

struct ServiceProcessMapDirect<
    Data,
    T,
    Mapper: Fn(Data) -> T,
    Process: ServiceProcess<Data> + Unpin,
> {
    process: Process,
    mapper: Mapper,
    id: Id,
    phantom_data: std::marker::PhantomData<Data>,
    phantom_t: std::marker::PhantomData<T>,
}

impl<Data, T, Mapper, Process> ServiceProcessMap<T>
    for ServiceProcessMapDirect<Data, T, Mapper, Process>
where
    T: Send,
    Data: Send,
    Mapper: Fn(Data) -> T + Send,
    Process: ServiceProcess<Data> + Unpin + Send,
{
    fn stop(self: Box<Self>) {
        self.process.stop();
    }

    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<T>> {
        let ret = self.process.poll_next_unpin(cx);
        match ret {
            Poll::Ready(Some(data)) => Poll::Ready(Some((self.mapper)(data))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.process.size_hint()
    }

    fn id(&self) -> Id {
        self.id
    }

    fn setup(&mut self, mailbox: ServiceMailbox) {
        self.process.setup(mailbox);
    }
}

struct ServiceProcessMapped<T, U, Mapper: Fn(T) -> U> {
    inner: Box<dyn ServiceProcessMap<T>>,
    mapper: Arc<Mapper>,
}

impl<T, U, Mapper: Fn(T) -> U + Send + Sync> ServiceProcessMap<U> for ServiceProcessMapped<T, U, Mapper> {
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

    fn setup(&mut self, mailbox: ServiceMailbox) {
        self.inner.setup(mailbox)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TxServiceMessage {
    Frontend(Vec<u8>),
    RunJs(String)
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RxServiceMessage {
    Frontend(Vec<u8>),
}

pub struct ServiceMailbox {
    rx: UnboundedReceiver<TxServiceMessage>,
    tx: UnboundedSender<RxServiceMessage>,
}
