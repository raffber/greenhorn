use crate::dom_event::DomEvent;
use crate::event::{Emission, Event};
use crate::service::{Service, ServiceSubscription};
use std::any::Any;
use std::marker::PhantomData;
use std::sync::mpsc::{channel, Receiver, Sender, RecvError};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use std::future::Future;
use futures::{Stream, StreamExt};
use std::pin::Pin;
use crate::dialog::{Dialog, DialogBinding};

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

pub struct Context<T: 'static + Send> {
    tx: MapSender<ContextMsg<T>>,
}

impl<T: 'static + Send> Clone for Context<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

pub(crate) struct MailboxReceiver<T: 'static + Send> {
    pub(crate) rx: Receiver<ContextMsg<T>>,
}

impl<T: 'static + Send> MailboxReceiver<T> {
    fn recv(&self) -> Result<ContextMsg<T>, RecvError> {
        self.rx.recv()
    }
}

pub(crate) enum ContextMsg<T: 'static + Send> {
    Emission(Emission),
    LoadCss(String),
    RunJs(String),
    Propagate(EventPropagate),
    Subscription(ServiceSubscription<T>),
    Future(Pin<Box<dyn Send + Future<Output=T>>>, bool), // (future, blocking)
    Stream(Pin<Box<dyn Send + Stream<Item=T>>>),
    Dialog(DialogBinding<T>),
}

impl<T: Send + 'static> ContextMsg<T> {
    pub fn map<U, Mapper>(self, mapper: Arc<Mapper>) -> ContextMsg<U>
    where
        U: 'static + Send,
        Mapper: 'static + Fn(T) -> U + Send + Sync
    {
        match self {
            ContextMsg::Subscription(subs) => ContextMsg::Subscription(subs.map(mapper)),
            ContextMsg::Future(fut, blocking) => {
                ContextMsg::Future(Box::pin(async move {
                    (mapper)(fut.await)
                }), blocking)
            },
            ContextMsg::Stream(stream) => {
                ContextMsg::Stream(Box::pin(stream.map(move |x| (mapper)(x))))
            },
            ContextMsg::Dialog(d) => {
                ContextMsg::Dialog(d.map(mapper))
            }
            _ => panic!()
        }
    }
}

impl<T: Send + 'static> Context<T> {
    pub(crate) fn new() -> (Self, MailboxReceiver<T>) {
        let (tx, rx) = channel();
        (
            Context {
                tx: MapSender::new(tx),
            },
            MailboxReceiver {
                rx,
            },
        )
    }

    pub fn emit<D: Any>(&self, event: &Event<D>, data: D) {
        let emission = event.emit(data);
        self.tx.send(ContextMsg::Emission(emission));
    }

    pub fn load_css<Css: Into<String>>(&self, css: Css) {
        self.tx.send(ContextMsg::LoadCss(css.into()));
    }

    pub fn run_js<Js: Into<String>>(&self, js: Js) {
        self.tx.send(ContextMsg::RunJs(js.into()));
    }

    pub fn run_service<S, F>(&self, service: S, fun: F)
    where
        S: Service + Send + Unpin + 'static,
        T: Send,
        F: 'static + Fn(S::Data) -> T + Send,
    {
        let subs = ServiceSubscription::new(service, fun);
        self.tx.send(ContextMsg::Subscription(subs));
    }

    pub fn spawn<Fut: 'static + Send + Future<Output=T>>(&self, fut: Fut) {
        self.tx.send(ContextMsg::Future(Box::pin(fut), false));
    }

    pub fn spawn_blocking<Fut: 'static + Send + Future<Output=T>>(&self, fut: Fut) {
        self.tx.send(ContextMsg::Future(Box::pin(fut), true));
    }

    pub fn subscribe<S: 'static + Send + Stream<Item=T>>(&self, stream: S) {
        self.tx.send(ContextMsg::Stream(Box::pin(stream)));
    }

    pub fn map<U: Send + 'static, F: 'static + Send + Sync + Fn(U) -> T>(
        &self,
        fun: F,
    ) -> Context<U> {
        let mapper = Arc::new(fun);
        let new_sender = self.tx.clone();
        let mapped = new_sender.map(move |msg: ContextMsg<U>| msg.map(mapper.clone()));
        Context {
            tx: mapped,
        }
    }

    pub fn propagate(&self, e: DomEvent) {
        self.tx
            .send(ContextMsg::Propagate(EventPropagate {
                event: e,
                propagate: true,
                default_action: false,
            }));
    }

    pub fn default_action(&self, e: DomEvent) {
        self.tx
            .send(ContextMsg::Propagate(EventPropagate {
                event: e,
                propagate: false,
                default_action: true,
            }));
    }

    pub fn propagate_and_default(&self, e: DomEvent) {
        self.tx
            .send(ContextMsg::Propagate(EventPropagate {
                event: e,
                propagate: true,
                default_action: true,
            }));
    }

    pub fn dialog<D: 'static + Dialog, F: 'static + Fn(D::Msg) -> T>(&self, dialog: D, fun: F) {
        let binding = DialogBinding::new(dialog, fun);
        self.tx.send(ContextMsg::Dialog(binding));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::tests::MsgB::ItemB;
    use assert_matches::assert_matches;
    use futures::task::{Context, Poll};
    use futures::{Stream, StreamExt};
    use std::pin::Pin;
    use crate::service::ServiceMailbox;
    use crate::context::ContextMsg::Subscription;
    use crate::context::tests::MsgA::ItemA;

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
        let (mb, rx) = Context::<MsgA>::new();
        let mapped = mb.map(MsgA::ItemA);
        let service = MyService {};
        mapped.run_service(service, ItemB);
        if let Ok(Subscription(mut subs)) = rx.recv() {
            let result = async_std::task::block_on(subs.next());
            assert_matches!(result, Some(MsgA::ItemA(MsgB::ItemB(1))));
        } else {
            panic!();
        }
    }

    #[test]
    fn test_future() {
        let fut = async {
            MsgB::ItemB(123)
        };

        let (mb, rx) = Context::<MsgA>::new();
        let mapped = mb.map(MsgA::ItemA);
        mapped.spawn(fut);
        if let Ok(ContextMsg::Future(fut)) = rx.recv() {
            let result = async_std::task::block_on(fut);
            assert_matches!(result, MsgA::ItemA(MsgB::ItemB(123)));
        } else {
            panic!();
        }
    }

    #[test]
    fn test_stream() {
        let stream = futures::stream::unfold(0, |state| async move {
            if state <= 2 {
                let next_state = state + 1;
                Some((MsgB::ItemB(state), next_state))
            } else {
                None
            }
        });
        let (mb, rx) = Context::<MsgA>::new();
        let mapped = mb.map(MsgA::ItemA);
        mapped.subscribe(stream);
        if let Ok(ContextMsg::Stream(stream)) = rx.recv() {
            let data: Vec<MsgA> = async_std::task::block_on(stream.collect::<Vec<MsgA>>());
            assert_eq!(data.len(), 3);
            assert_matches!(data[0], ItemA(ItemB(0)));
            assert_matches!(data[1], ItemA(ItemB(1)));
            assert_matches!(data[2], ItemA(ItemB(2)));
        } else {
            panic!();
        }
    }
}
