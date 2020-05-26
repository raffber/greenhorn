//! This module implements an executor for [Service](../trait.Service.html) objects.
//!
//! It spawns a new task and feeds the update messages emitted by the service
//! back into the `update()` cycle of the application.

use crate::service::{RxServiceMessage, ServiceSubscription, TxServiceMessage};
use crate::Id;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::task::{Context, Poll};
use futures::{Stream, StreamExt};
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::pin::Pin;
use futures::stream;

/// Message type used to communicate between ServiceRunner and ServiceCollection.
pub(crate) enum ServiceMessage<Msg> {
    Update(Msg),
    Tx(Id, TxServiceMessage),
    Stopped(Id),
}

impl<Msg: Debug> Debug for ServiceMessage<Msg> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            ServiceMessage::Update(msg) => {
                f.write_fmt(format_args!("ServiceMessage::Update({:?})", msg))
            }
            ServiceMessage::Tx(id, msg) => {
                f.write_fmt(format_args!("ServiceMessage::Tx({:?}, {:?})", id, msg))
            }
            ServiceMessage::Stopped(id) => {
                f.write_fmt(format_args!("ServiceMessage::Stopped({:?})", id))
            }
        }
    }
}

/// A registry of services that are being executed
///
/// Note that this type is also a `Stream`. All messages
/// from spawned services are merged in this stream.
pub(crate) struct ServiceCollection<Msg> {
    services: HashMap<Id, ServiceControl>,
    msg_receiver: UnboundedReceiver<ServiceMessage<Msg>>,
    msg_sender: UnboundedSender<ServiceMessage<Msg>>,
}

impl<Msg> Stream for ServiceCollection<Msg> {
    type Item = ServiceMessage<Msg>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let ret = Pin::new(&mut self.msg_receiver).poll_next(cx);
        match ret {
            Poll::Ready(Some(ServiceMessage::Stopped(id))) => {
                self.services.remove(&id);
                Poll::Ready(Some(ServiceMessage::Stopped(id)))
            }
            x => x,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.msg_receiver.size_hint()
    }
}

impl<Msg: Send> ServiceCollection<Msg> {
    pub(crate) fn new() -> Self {
        let (tx, rx) = unbounded();
        Self {
            services: HashMap::new(),
            msg_receiver: rx,
            msg_sender: tx,
        }
    }

    /// Start executing a new service on this `ServiceCollection`.
    pub(crate) fn spawn(&mut self, subs: ServiceSubscription<Msg>) {
        let id = subs.id();
        let mailbox_tx = subs.rxmailbox_tx.clone();
        let runner = ServiceRunner {
            tx: self.msg_sender.clone(),
            service: subs,
        };
        runner.run();
        let control = ServiceControl { mailbox_tx };
        self.services.insert(id, control);
    }

    /// Stop all running services
    fn stop_all(mut self) {
        self.services.drain().for_each(|x| x.1.stop());
        self.msg_receiver.close();
    }

    /// Send a message to a service identified by `id`.
    pub(crate) fn send(&mut self, id: Id, msg: RxServiceMessage) {
        if let Some(x) = self.services.get(&id) {
            if x.mailbox_tx.unbounded_send(msg).is_err() {
                // the service has terminated
                self.services.remove(&id);
            }
        }
    }
}

/// Control handle for a running service
struct ServiceControl {
    mailbox_tx: UnboundedSender<RxServiceMessage>,
}

impl ServiceControl {
    /// Send a stop message to the service.
    fn stop(&self) {
        let _ = self.mailbox_tx.unbounded_send(RxServiceMessage::Stop);
    }
}

/// An executor of a single service.
/// It manages the lifecycle of the stream.
pub struct ServiceRunner<Msg: 'static + Send> {
    tx: UnboundedSender<ServiceMessage<Msg>>,
    service: ServiceSubscription<Msg>,
}

/// Used to merge streams
enum ServiceRunnerMsg<Msg: Send> {
    Tx(TxServiceMessage),
    Msg(Msg),
}

impl<Msg: Send> ServiceRunner<Msg> {
    /// Spawn a new task and run the contained service in it.
    pub(crate) fn run(self) {
        let runner = self;
        crate::platform::spawn(async {
            let id = runner.service.id();
            let mut service = runner.service;
            let txmailbox_rx = service.txmailbox_rx.take().unwrap();
            let mut stream = stream::select(
                txmailbox_rx.map(ServiceRunnerMsg::Tx),
                StreamExt::map(&mut service, ServiceRunnerMsg::Msg));
            while let Some(msg) = stream.next().await {
                match msg {
                    ServiceRunnerMsg::Tx(tx_msg) => {
                        if runner.tx.unbounded_send(ServiceMessage::Tx(id, tx_msg)).is_err() {
                            // runtime closed receiving end. Terminate service.
                            break
                        }
                    },
                    ServiceRunnerMsg::Msg(msg) => {
                        if runner.tx.unbounded_send(ServiceMessage::Update(msg)).is_err() {
                            // runtime closed receiving end. Terminate service.
                            break
                        }
                    }
                }
            }
            // notify the world that the service has stopped.
            // if the channel is already broken, the receiving ends have probably hung up
            // this is no big deal, we can just ignore this condition.
            let _ = runner.tx.unbounded_send(ServiceMessage::Stopped(id));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::{Mailbox, Service};
    use assert_matches::assert_matches;
    use async_std::task;

    struct DummyService;

    impl Service for DummyService {
        type Data = i32;
        type DataStream = UnboundedReceiver<i32>;

        fn start(self, _mailbox: Mailbox) -> Self::DataStream {
            let (tx, rx) = unbounded();
            task::spawn(async move {
                for k in 0..4_i32 {
                    tx.unbounded_send(k).unwrap();
                }
            });
            rx
        }
    }

    #[test]
    fn service_runner_without_frontend_io() {
        let subs = ServiceSubscription::new(DummyService, |x| x);
        let mut col = ServiceCollection::new();
        let id = subs.id();
        col.spawn(subs);
        task::block_on(async move {
            assert_matches!(col.next().await, Some(ServiceMessage::Update(0)));
            assert_matches!(col.next().await, Some(ServiceMessage::Update(1)));
            assert_matches!(col.next().await, Some(ServiceMessage::Update(2)));
            assert_matches!(col.next().await, Some(ServiceMessage::Update(3)));
            match col.next().await {
                Some(ServiceMessage::Stopped(x)) => {
                    assert_eq!(x, id);
                }
                _ => panic!("Expected service stop now."),
            }
            // check if service was freed
            assert_eq!(col.services.len(), 0);
            // stop service collection
            col.stop_all();
        });
    }

    struct IoService;

    impl Service for IoService {
        type Data = i32;
        type DataStream = UnboundedReceiver<i32>;

        fn start(self, mailbox: Mailbox) -> Self::DataStream {
            let (tx, rx) = unbounded();
            task::spawn(async move {
                mailbox.run_js("foo");
                for k in 0..4_i32 {
                    mailbox.run_js("foo");
                    tx.unbounded_send(k).unwrap();
                }
                mailbox.run_js("foo");
            });
            rx
        }
    }

    fn check_js_msg(msg: Option<ServiceMessage<i32>>, id: Id) {
        match msg {
            Some(ServiceMessage::Tx(msg_id, msg)) => match msg {
                TxServiceMessage::RunJs(msg) => {
                    assert_eq!(msg_id, id);
                    assert_eq!(msg, "foo");
                }
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn service_runner_with_frontend_io() {
        let subs = ServiceSubscription::new(IoService, |x| x);
        let mut col = ServiceCollection::new();
        let subs_id = subs.id();
        col.spawn(subs);
        task::block_on(async move {
            let mut expected_msg = 0;
            let mut js_count = 0;
            while let Some(x) = col.next().await {
                match x {
                    ServiceMessage::Update(k) => {
                        assert_eq!(k, expected_msg);
                        if k > 4 {
                            panic!();
                        }
                        expected_msg += 1;
                    }
                    ServiceMessage::Tx(id, TxServiceMessage::RunJs(x)) => {
                        assert_eq!(x, "foo");
                        assert_eq!(id, subs_id);
                        js_count += 1;
                    }
                    ServiceMessage::Stopped(id) => {
                        assert_eq!(id, subs_id);
                        break;
                    }
                    _ => panic!(),
                }
            }
            assert_eq!(expected_msg, 4);
            assert_eq!(js_count, 6);
            col.stop_all();
        });
    }
}
