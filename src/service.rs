//! This module exposes the [Service](trait.Service.html) trait, which allows writing custom agents
//! communicating with the frontend.
//!

use futures::task::{Context, Poll};
use futures::{Stream, StreamExt};

use crate::Id;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::Arc;

/// The `Service` trait allows a type to act as an agent, which communicates over custom messages
/// with the frontend and injects messages back into the `update()` cycle of the application.
///
/// After calling the `start()` method, a services produces `Data` items which are injected back
/// into the `update()` cycle of the application.
/// The service itself can send and receive messages from the frontend
/// with the [Mailbox](struct.Mailbox.html)
/// object it has received with the `start()` call.
///
/// # Example
///
/// ```
/// # use greenhorn::service::{Service, Mailbox, RxServiceMessage};
/// # use std::str::FromStr;
/// use futures::{channel::mpsc, StreamExt};
/// use greenhorn::platform;
///
/// struct MyService {}
///
/// impl Service for MyService {
///    type Data = i32;
///    type DataStream = mpsc::UnboundedReceiver<i32>;
///
///    fn start(self, mut mailbox: Mailbox) -> Self::DataStream {
///        mailbox.run_js("ctx.send('123')");
///        let (tx, rx) = mpsc::unbounded();
///        platform::spawn(async move {
///            loop {
///                if let Some(RxServiceMessage::Frontend(data)) = mailbox.next().await {
///                    // this will receive '123' from the frontend
///                    let data = i32::from_str(&data).unwrap();
///                    // send the i32 into the update() cycle of the application
///                    tx.unbounded_send(data).unwrap();
///                } else {
///                    break;
///                }
///            }
///        });
///        rx
///    }
///}
/// ```
pub trait Service {
    /// The data item the service emits.
    ///
    /// These items are wrapped in messages and passed to the `update()` cycle of the application
    type Data: 'static + Send;

    /// A stream which produces the services data items
    type DataStream: Stream<Item = <Self as Service>::Data> + Send;

    /// Starts a service and consumes it.
    ///
    /// The returned `DataStream` may produce items to be passed into the `update()` loop of the
    /// applications.
    /// The service should subscribe to messages from the [Mailbox](struct.Mailbox.html) (a `Stream`).
    fn start(self, mailbox: Mailbox) -> Self::DataStream;
}

/// A wrapper for a polymorphic stream
///
/// This is a bit redundant with `futures::BoxStream`
struct BoxedStream<T: 'static + Send> {
    inner: Pin<Box<dyn Stream<Item = T> + Send>>,
}

impl<T: 'static + Send> BoxedStream<T> {
    fn new<U: 'static + Stream<Item = T> + Send>(stream: U) -> Self {
        Self {
            inner: Box::pin(stream),
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

/// Holds a started service, its meta-data and channels to control it
pub(crate) struct ServiceSubscription<T: 'static + Send> {
    inner: BoxedStream<T>,
    id: Id,
    pub(crate) rxmailbox_tx: UnboundedSender<RxServiceMessage>,
    pub(crate) txmailbox_rx: Option<UnboundedReceiver<TxServiceMessage>>,
}

impl<T: 'static + Send> ServiceSubscription<T> {
    /// Start a new service and subscribe to it
    pub(crate) fn new<Data, DataStream, S, Fun>(service: S, fun: Fun) -> Self
    where
        Data: 'static + Send,
        DataStream: 'static + Stream<Item = Data> + Send,
        S: Service<Data = Data, DataStream = DataStream>,
        Fun: 'static + Send + Fn(Data) -> T,
    {
        let (txmailbox_tx, txmailbox_rx) = unbounded::<TxServiceMessage>();
        let (rxmailbox_tx, rxmailbox_rx) = unbounded::<RxServiceMessage>();
        let mailbox = Mailbox {
            rx: rxmailbox_rx,
            tx: txmailbox_tx,
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
        Mapper: 'static + Fn(T) -> U + Send + Sync,
    {
        let fun = move |x| (*mapper)(x);
        let inner = self.inner.map(fun);
        let ret = BoxedStream::new(inner);
        ServiceSubscription {
            inner: ret,
            id: self.id,
            rxmailbox_tx: self.rxmailbox_tx,
            txmailbox_rx: self.txmailbox_rx,
        }
    }

    pub(crate) fn id(&self) -> Id {
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

/// TxServiceMessage objects a service may send to the frontend
///
/// Messages can be sent using the [Mailbox](struct.Mailbox.html) type.
#[derive(Serialize, Deserialize, Debug)]
pub enum TxServiceMessage {
    Frontend(Vec<u8>),
    RunJs(String),
    LoadCss(String),
}

/// TxServiceMessage objects a service receives from the frontend
#[derive(Serialize, Deserialize, Debug)]
pub enum RxServiceMessage {
    Frontend(String),
    Stop,
}

/// The mailbox type allows a [Service](trait.Service.html) to communicate with the frontend via
/// custom messages.
///
/// Note that this type implements `Stream` with `Item = RxServiceMessage`.
pub struct Mailbox {
    pub(crate) rx: UnboundedReceiver<RxServiceMessage>,
    pub(crate) tx: UnboundedSender<TxServiceMessage>,
}

impl Mailbox {
    /// Run a piece of js code on the frontend
    ///
    /// TODO: document ctx js object
    /// TODO: add an example
    pub fn run_js<T: Into<String>>(&self, code: T) {
        let _ = self.tx.unbounded_send(TxServiceMessage::RunJs(code.into()));
    }

    /// Send data to the service on the frontend
    pub fn send_data(&self, data: Vec<u8>) {
        let _ = self.tx.unbounded_send(TxServiceMessage::Frontend(data));
    }

    /// Load CSS on the frontend
    pub fn load_css<T: Into<String>>(&self, css: T) {
        let _ = self
            .tx
            .unbounded_send(TxServiceMessage::LoadCss(css.into()));
    }
}

impl Stream for Mailbox {
    type Item = RxServiceMessage;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.rx).poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.rx.size_hint()
    }
}
