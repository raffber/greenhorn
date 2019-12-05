use crate::dom_event::DomEvent;
use crate::event::{Emission, Event};
use crate::service::{Service, ServiceSubscription};
use std::any::Any;
use std::marker::PhantomData;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

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

pub(crate) struct EventPropagate {
    event: DomEvent,
    propagate: bool,
    default_action: bool,
}

pub struct Mailbox<T: 'static> {
    tx: Sender<MailboxMsg>,
    services: MapSender<ServiceSubscription<T>>,
}

impl<T: 'static> Clone for Mailbox<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            services: self.services.clone(),
        }
    }
}

pub(crate) struct MailboxReceiver<T: 'static> {
    pub(crate) rx: Receiver<MailboxMsg>,
    pub(crate) services: Receiver<ServiceSubscription<T>>,
}

pub(crate) enum MailboxMsg {
    Emission(Emission),
    LoadCss(String),
    RunJs(String),
    Propagate(EventPropagate),
}

impl<T: 'static> Mailbox<T> {
    pub(crate) fn new() -> (Self, MailboxReceiver<T>) {
        let (tx, rx) = channel();
        let (s_tx, s_rx) = channel();
        (
            Mailbox {
                tx,
                services: MapSender::new(s_tx),
            },
            MailboxReceiver {
                rx,
                services: s_rx,
            },
        )
    }

    pub fn emit<D: Any>(&self, event: Event<D>, data: D) {
        let emission = event.emit(data);
        self.tx.send(MailboxMsg::Emission(emission)).unwrap();
    }

    pub fn load_css<Css: Into<String>>(&self, css: Css) {
        self.tx.send(MailboxMsg::LoadCss(css.into())).unwrap();
    }

    pub fn spawn<S, F>(&self, service: S, fun: F)
    where
        S: Service + Send + Unpin + 'static,
        T: Send,
        F: 'static + Fn(S::Data) -> T + Send,
    {
        let subs = ServiceSubscription::new(service, fun);
        self.services.send(subs);
    }

    pub fn map<U: Send + 'static, F: 'static + Send + Sync + Fn(U) -> T>(
        &self,
        fun: F,
    ) -> Mailbox<U> {
        let mapper = Arc::new(fun);
        let new_sender = self.services.clone();
        let mapped = new_sender.map(move |subs: ServiceSubscription<U>| subs.map(mapper.clone()));
        Mailbox {
            tx: self.tx.clone(),
            services: mapped,
        }
    }

    pub fn propagate(&self, e: DomEvent) {
        self.tx
            .send(MailboxMsg::Propagate(EventPropagate {
                event: e,
                propagate: true,
                default_action: false,
            }))
            .unwrap();
    }

    pub fn default_action(&self, e: DomEvent) {
        self.tx
            .send(MailboxMsg::Propagate(EventPropagate {
                event: e,
                propagate: false,
                default_action: true,
            }))
            .unwrap();
    }

    pub fn propagate_and_default(&self, e: DomEvent) {
        self.tx
            .send(MailboxMsg::Propagate(EventPropagate {
                event: e,
                propagate: true,
                default_action: true,
            }))
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mailbox::tests::MsgB::ItemB;
    use assert_matches::assert_matches;
    use dummy_waker::dummy_context;
    use futures::task::{Context, Poll};
    use futures::Stream;
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
    fn test_mailbox() {
        let ctx = dummy_context();
        let (mb, rx) = Mailbox::<MsgA>::new();
        let mapped = mb.map(MsgA::ItemA);
        let service = MyService {};
        mapped.spawn(service, ItemB);
        if let Ok(mut subs) = rx.services.recv() {
            let polled = Pin::new(&mut subs).poll_next(&mut ctx.context());
            assert_matches!(polled, Poll::Ready(Some(MsgA::ItemA(MsgB::ItemB(1)))));
        } else {
            panic!();
        }
    }
}
