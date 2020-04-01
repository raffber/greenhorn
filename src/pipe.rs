use futures::Stream;

use crate::dom_event::DomEvent;
use crate::service::{RxServiceMessage, TxServiceMessage};
use serde::{Deserialize, Serialize};
use crate::mailbox::EventPropagate;
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

// TODO: make macro work
//trait_alias!(Receiver = Stream<Item=RxMsg> + Unpin + Send + 'static);

pub trait Pipe {
    type Sender: Sender;
    type Receiver: Receiver;

    fn split(self) -> (Self::Sender, Self::Receiver);
}
