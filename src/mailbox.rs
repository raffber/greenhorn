use crate::dom_event::DomEvent;
use crate::event::{Emission, Event};
use crate::service::{Service, ServiceSubscription};
use std::any::Any;
use std::marker::PhantomData;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use std::future::Future;
use futures::{Stream, StreamExt};
use std::pin::Pin;

enum MapSender<T> {
    Direct(Sender<T>),
    Mapped(Box<dyn MappedSender<T>>),
}

impl<T> Clone for MapSender<T> {
    fn clone(&self) -> Self {
        match self {
            MapSender::Direct(sender) => MapSender::Direct(sender.clone()),
            MapSender::Mapped(mapped) => MapSender::Mapped(mapped.clone_box()),
        }
    }
}

impl<T: 'static> MapSender<T> {
    fn new(sender: Sender<T>) -> Self {
        MapSender::Direct(sender)
    }

    fn map<U: 'static, Mapper: 'static + Fn(U) -> T>(self, mapper: Mapper) -> MapSender<U> {
        MapSender::Mapped(Box::new(MappedSenderImpl {
            mapper: Arc::new(mapper),
            sender: self,
            phantom: PhantomData,
        }))
    }

    fn send(&self, value: T) {
        match self {
            MapSender::Direct(sender) => {
                sender.send(value).unwrap();
            }
            MapSender::Mapped(mapped) => mapped.send(value),
        }
    }
}

trait MappedSender<T> {
    fn send(&self, value: T);
    fn clone_box(&self) -> Box<dyn MappedSender<T>>;
}

struct MappedSenderImpl<T, U, Mapper: Fn(U) -> T> {
    mapper: Arc<Mapper>,
    sender: MapSender<T>,
    phantom: PhantomData<U>,
}

impl<T: 'static, U: 'static, Mapper: 'static + Fn(U) -> T> MappedSender<U>
    for MappedSenderImpl<T, U, Mapper>
{
    fn send(&self, value: U) {
        let value = (self.mapper)(value);
        self.sender.send(value);
    }

    fn clone_box(&self) -> Box<dyn MappedSender<U>> {
        Box::new(MappedSenderImpl {
            mapper: self.mapper.clone(),
            sender: self.sender.clone(),
            phantom: PhantomData,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventPropagate {
    event: DomEvent,
    propagate: bool,
    default_action: bool,
}

pub struct Mailbox<T: 'static + Send> {
    tx: MapSender<MailboxMsg<T>>,
}

impl<T: 'static + Send> Clone for Mailbox<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

pub(crate) struct MailboxReceiver<T: 'static + Send> {
    pub(crate) rx: Receiver<MailboxMsg<T>>,
}

pub(crate) enum MailboxMsg<T: 'static + Send> {
    Emission(Emission),
    LoadCss(String),
    RunJs(String),
    Propagate(EventPropagate),
    Subscription(ServiceSubscription<T>),
    Future(Pin<Box<dyn Send + Future<Output=T>>>),
    Stream(Pin<Box<dyn Send + Stream<Item=T>>>),
}

impl<T: Send + 'static> MailboxMsg<T> {
    pub fn map<U, Mapper>(self, mapper: Arc<Mapper>) -> MailboxMsg<U>
    where
        U: 'static + Send,
        Mapper: 'static + Fn(T) -> U + Send + Sync
    {
        match self {
            MailboxMsg::Subscription(subs) => MailboxMsg::Subscription(subs.map(mapper)),
            MailboxMsg::Future(fut) => {
                MailboxMsg::Future(Box::pin(async move {
                    (mapper)(fut.await)
                }))
            },
            MailboxMsg::Stream(stream) => {
                MailboxMsg::Stream(Box::pin(stream.map(move |x| (mapper)(x))))
            },
            _ => panic!()
        }
    }
}

impl<T: Send + 'static> Mailbox<T> {
    pub(crate) fn new() -> (Self, MailboxReceiver<T>) {
        let (tx, rx) = channel();
        (
            Mailbox {
                tx: MapSender::new(tx),
            },
            MailboxReceiver {
                rx,
            },
        )
    }

    pub fn emit<D: Any>(&self, event: &Event<D>, data: D) {
        let emission = event.emit(data);
        self.tx.send(MailboxMsg::Emission(emission));
    }

    pub fn load_css<Css: Into<String>>(&self, css: Css) {
        self.tx.send(MailboxMsg::LoadCss(css.into()));
    }

    pub fn run_js<Js: Into<String>>(&self, js: Js) {
        self.tx.send(MailboxMsg::RunJs(js.into()));
    }

    pub fn run_service<S, F>(&self, service: S, fun: F)
    where
        S: Service + Send + Unpin + 'static,
        T: Send,
        F: 'static + Fn(S::Data) -> T + Send,
    {
        let subs = ServiceSubscription::new(service, fun);
        self.tx.send(MailboxMsg::Subscription(subs));
    }

    pub fn spawn<Fut: 'static + Send + Future<Output=T>>(&self, fut: Fut) {
        self.tx.send(MailboxMsg::Future(Box::pin(fut)));
    }

    pub fn subscribe<S: 'static + Send + Stream<Item=T>>(&self, stream: S) {
        self.tx.send(MailboxMsg::Stream(Box::pin(stream)));
    }

    pub fn map<U: Send + 'static, F: 'static + Send + Sync + Fn(U) -> T>(
        &self,
        fun: F,
    ) -> Mailbox<U> {
        let mapper = Arc::new(fun);
        let new_sender = self.tx.clone();
        let mapped = new_sender.map(move |msg: MailboxMsg<U>| msg.map(mapper.clone()));
        Mailbox {
            tx: mapped,
        }
    }

    pub fn propagate(&self, e: DomEvent) {
        self.tx
            .send(MailboxMsg::Propagate(EventPropagate {
                event: e,
                propagate: true,
                default_action: false,
            }));
    }

    pub fn default_action(&self, e: DomEvent) {
        self.tx
            .send(MailboxMsg::Propagate(EventPropagate {
                event: e,
                propagate: false,
                default_action: true,
            }));
    }

    pub fn propagate_and_default(&self, e: DomEvent) {
        self.tx
            .send(MailboxMsg::Propagate(EventPropagate {
                event: e,
                propagate: true,
                default_action: true,
            }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mailbox::tests::MsgB::ItemB;
    use assert_matches::assert_matches;
    use futures::task::{Context, Poll};
    use futures::{Stream, StreamExt};
    use std::pin::Pin;
    use crate::service::ServiceMailbox;

    #[derive(Debug)]
    enum MsgA {
        ItemA(MsgB),
    }

    #[derive(Debug)]
    enum MsgB {
        ItemB(i32),
    }

    struct MyService {}

    impl Service for MyService {
        type Data = i32;

        fn start(&mut self, _mailbox: ServiceMailbox) {
        }

        fn stop(self) {
        }
    }

    impl Stream for MyService {
        type Item = i32;

        fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            Poll::Ready(Some(1))
        }
    }

    #[test]
    fn test_service() {
        let (mb, rx) = Mailbox::<MsgA>::new();
        let mapped = mb.map(MsgA::ItemA);
        let service = MyService {};
        mapped.run_service(service, ItemB);
        if let Ok(mut subs) = rx.rx.recv() {
            let result = async_std::task::block_on(subs.next());
            assert_matches!(result, Some(MsgA::ItemA(MsgB::ItemB(1))));
        } else {
            panic!();
        }
    }
//
//    #[test]
//    fn test_future() {
//        let fut = async {
//            MsgB::ItemB(123)
//        };
//
//        let (mb, rx) = Mailbox::<MsgA>::new();
//        let mapped = mb.map(MsgA::ItemA);
//        mapped.spawn(fut);
//        if let Ok(mut fut) = rx.rx
//        if let Ok(mut )
//    }
}
