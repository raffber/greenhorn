use futures::Stream;
use futures::Sink;

use crate::dom_event::DomEvent;
use crate::service::{RxServiceMessage, TxServiceMessage};
use serde::{Deserialize, Serialize};
use crate::context::EventPropagate;
use serde_json::Value as JsonValue;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub enum TxMsg {
    Ping(),
    Patch(Vec<u8>),
    LoadCss(String),
    RunJs(String),
    Service(u64, TxServiceMessage),
    Propagate(EventPropagate),
    Dialog(JsonValue),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RxMsg {
    Event(DomEvent),
    FrameApplied(),
    Service(u64, RxServiceMessage),
    Dialog(JsonValue),
}

pub trait Receiver: Stream<Item = RxMsg> + Unpin + Send + 'static {}
impl<T> Receiver for T where T: Stream<Item = RxMsg> + Unpin + Send + 'static {}

pub trait Sender: Sink<TxMsg, Error=Box<dyn Error>> + Unpin + Send + Clone + 'static {}
impl<T> Sender for T where T: Sink<TxMsg, Error=Box<dyn Error>> + Unpin + Send + Clone + 'static {}


pub trait Pipe {
    type Sender: Sender;
    type Receiver: Receiver;

    fn split(self) -> (Self::Sender, Self::Receiver);
}


#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use futures::task::{Context, Poll};
    use std::pin::Pin;
    use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};

    #[derive(Clone)]
    pub(crate) struct DummySender(UnboundedSender<TxMsg>);

    impl Sink<TxMsg> for DummySender {
        type Error = Box<dyn Error>;

        fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Pin::new(&mut self.0).poll_ready(cx).map_err(|x| Box::new(x).into())
        }

        fn start_send(mut self: Pin<&mut Self>, item: TxMsg) -> Result<(), Self::Error> {
            Pin::new(&mut self.0).start_send(item).map_err(|x| Box::new(x).into())
        }

        fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Pin::new(&mut self.0).poll_flush(cx).map_err(|x| Box::new(x).into())
        }

        fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Pin::new(&mut self.0).poll_close(cx).map_err(|x| Box::new(x).into())
        }
    }

    pub(crate) struct DummyPipe {
        sender_tx: UnboundedSender<TxMsg>,
        receiver_rx: UnboundedReceiver<RxMsg>,
    }

    pub(crate) struct DummyFrontend {
        pub(crate) sender_rx: UnboundedReceiver<TxMsg>,
        pub(crate) receiver_tx: UnboundedSender<RxMsg>,
    }

    impl DummyPipe {
        pub(crate) fn new() -> (Self, DummyFrontend) {
            let (sender_tx, sender_rx) = unbounded();
            let (receiver_tx, receiver_rx) = unbounded();
            (Self {
                sender_tx,
                receiver_rx,
            }, DummyFrontend {
                sender_rx,
                receiver_tx
            })
        }
    }

    impl Pipe for DummyPipe {
        type Sender = DummySender;
        type Receiver = Box<dyn Receiver>;

        fn split(self) -> (Self::Sender, Self::Receiver) {
            let sender_tx = DummySender(self.sender_tx);
            let receiver_rx: Box<dyn Receiver> = Box::new(self.receiver_rx);
            (sender_tx, receiver_rx)
        }
    }
}
