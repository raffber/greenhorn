use futures::Stream;

use crate::dom_event::DomEvent;
use serde::{Deserialize, Serialize};
use crate::service::{TxServiceMessage, RxServiceMessage};

#[derive(Debug, Serialize, Deserialize)]
pub enum TxMsg {
    Ping(),
    Patch(Vec<u8>),
    Service(TxServiceMessage),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RxMsg {
    Ping(),
    Event(DomEvent),
    FrameApplied(),
    Service(RxServiceMessage)
}

pub trait Sender: Clone {
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
