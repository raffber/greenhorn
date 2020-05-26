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
/// After the `start()` method has been called, a service produces `Data` items which are injected back
/// into the `update()` cycle of the application.
/// The service itself can send and receive messages from the frontend
/// using the [Mailbox](struct.Mailbox.html)
/// object it has obtained from the invocation of `start()`.
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

/// A backend service may send `TxServiceMessage` objects to the frontend.
///
/// Messages can be sent using the [Mailbox](struct.Mailbox.html) type.
#[derive(Serialize, Deserialize, Debug)]
pub enum TxServiceMessage {
    RunJs(String),
    LoadCss(String),
}

/// `RxServiceMessage` objects are sent by the frontend and received by the backend.
///
/// Messages may be received using the [Mailbox](struct.Mailbox.html) struct, which
/// implements `Stream` with `Item = RxServiceMessage`.
#[derive(Serialize, Deserialize, Debug)]
pub enum RxServiceMessage {
    Frontend(String),
    Stop,
}

/// The mailbox type allows a [Service](trait.Service.html) to communicate with the frontend via
/// custom messages.
///
/// Note that this type implements `Stream` with `Item = RxServiceMessage`.
/// The frontend may communicate with the service by sending strings to the service.
/// These messages are emitted as `RxServiceMessage::Frontend(String)`.
/// In case the application shuts down, it will send the service `RxServiceMessage::Stop`.
/// This should usually lead to termination of the `Service`.
///
/// Refer to to [`Mailbox::run_js()`](struct.Mailbox.html#method.run_js) for detail on the
/// javascript API and how to send messages from the frontend to the backend.
///
pub struct Mailbox {
    pub(crate) rx: UnboundedReceiver<RxServiceMessage>,
    pub(crate) tx: UnboundedSender<TxServiceMessage>,
}

impl Mailbox {
    /// Run a piece of js code on the frontend.
    ///
    /// The javascript code has access to a `Context` object under the
    /// variable name `ctx`. This object has a `send(msg: String)` function
    /// which allows sending string from frontend to backend.
    /// The data can be received by handling messages emitted by the
    /// `Mailbox` object.
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
    /// // the javascript code has access to the `ctx` object
    /// // which has a `send()` function to pass data from the
    /// // frontend to the backend
    /// const JS: &str = r#"
    /// setInterval(function() {
    ///     ctx.send('123');
    /// }
    /// "#;
    ///
    /// impl Service for MyService {
    ///     type Data = i32;
    ///     type DataStream = mpsc::UnboundedReceiver<i32>;
    ///
    ///     fn start(self, mut mailbox: Mailbox) -> Self::DataStream {
    ///         mailbox.run_js(JS);
    ///         let (tx, rx) = mpsc::unbounded();
    ///         platform::spawn(async move {
    ///             loop {
    ///                 match mailbox.next().await {
    ///                     Some(RxServiceMessage::Frontend(data)) => {
    ///                         assert_eq!(data, "123");
    ///                         tx.unbounded_send(123).unwrap();
    ///                     },
    ///                     Some(RxServiceMessage::Stop) => break,
    ///                     None => break,
    ///                 }
    ///             }
    ///         });
    ///         rx
    ///     }
    /// }
    /// ```
    pub fn run_js<T: Into<String>>(&self, code: T) {
        let _ = self.tx.unbounded_send(TxServiceMessage::RunJs(code.into()));
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
