use futures::task::{Context, Poll};
use futures::{Stream, StreamExt};

use crate::Id;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::Arc;


pub trait Service {
    type Data: 'static + Send;
    type DataStream: Stream<Item = <Self as Service>::Data> + Send;

    fn start(self, mailbox: ServiceMailbox) -> Self::DataStream;
}

pub(crate) struct ServiceSubscription<T: 'static + Send>{
    inner: BoxedStream<T>,
    id: Id,
    pub(crate) rxmailbox_tx: UnboundedSender<RxServiceMessage>,
    pub(crate) txmailbox_rx: Option<UnboundedReceiver<TxServiceMessage>>,
}

// this is a bit redundant with futures::BoxStream
struct BoxedStream<T: 'static + Send> {
    inner: Pin<Box<dyn Stream<Item=T> + Send>>,
}

impl<T: 'static + Send> BoxedStream<T> {
    fn new<U: 'static + Stream<Item=T> + Send>(stream: U) -> Self {
        Self {
            inner: Box::pin(stream)
        }
    }
}

impl<T: 'static + Send> Stream for BoxedStream<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        Pin::new(&self.inner).size_hint()
    }
}

impl<T: 'static + Send> ServiceSubscription<T> {
    pub(crate) fn new<Data, DataStream, S, Fun>(service: S, fun: Fun) -> Self
        where
            Data: 'static + Send,
            DataStream: 'static + Stream<Item=Data> + Send,
            S: Service<Data=Data, DataStream=DataStream>,
            Fun: 'static + Send + Fn(Data) -> T,
    {
        let (txmailbox_tx, txmailbox_rx) = unbounded::<TxServiceMessage>();
        let (rxmailbox_tx, rxmailbox_rx) = unbounded::<RxServiceMessage>();
        let mailbox = ServiceMailbox {
            rx: rxmailbox_rx,
            tx: txmailbox_tx
        };
        let stream = service.start(mailbox);
        ServiceSubscription {
            inner: BoxedStream::new(stream.map(fun)),
            id: Default::default(),
            rxmailbox_tx,
            txmailbox_rx: Some(txmailbox_rx),
        }
    }

    pub(crate) fn map<U, Mapper>(self, mapper: Arc<Mapper>) -> ServiceSubscription<U>
        where
            U: 'static + Send,
            Mapper: 'static + Fn(T) -> U + Send + Sync
    {
        let fun = move |x| (*mapper)(x);
        let inner = self.inner.map(fun);
        let ret = BoxedStream::new(inner);
        ServiceSubscription {
            inner: ret,
            id: self.id,
            rxmailbox_tx: self.rxmailbox_tx,
            txmailbox_rx: self.txmailbox_rx
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }
}

impl<T: 'static + Send> Stream for ServiceSubscription<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        Pin::new(&self.inner).size_hint()
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
    Stop,
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
