use futures::Stream;

use crate::dom_event::DomEvent;
use crate::service::{RxServiceMessage, TxServiceMessage};
use serde::{Deserialize, Serialize};
use crate::context::EventPropagate;
use serde_json::Value as JsonValue;

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

pub trait Sender: Clone + Send {
    fn send(&self, msg: TxMsg);
    fn close(&self);
}

pub trait Receiver: Stream<Item = RxMsg> + Unpin + Send + 'static {}
impl<T> Receiver for T where T: Stream<Item = RxMsg> + Unpin + Send + 'static {}

// TODO: implement Sender for Sink?

// TODO: make macro work
//trait_alias!(Receiver = Stream<Item=RxMsg> + Unpin + Send + 'static);

pub trait Pipe {
    type Sender: Sender;
    type Receiver: Receiver;

    fn split(self) -> (Self::Sender, Self::Receiver);
}


#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use async_std::sync;
    use futures::task::{Context, Poll};
    use std::pin::Pin;
    use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};

    struct DummyPipe;

    struct DummySender;
    impl Sender for DummySender {
        fn send(&self, msg: TxMsg) { }
        fn close(&self) { }
    }

    impl Pipe for DummyPipe {
        type Sender = DummySender;
        type Receiver = UnboundedReceiver<RxMsg>;

        fn split(self) -> (Self::Sender, Self::Receiver) {

            unimplemented!()
        }
    }

    #[test]
    fn test_render_loop() {
        let mut component = DummyComponent(1);

    }

}
