use crate::service::{RxServiceMessage, ServiceSubscription, TxServiceMessage};
use crate::Id;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::select;
use futures::task::{Context, Poll};
use futures::{FutureExt, Stream, StreamExt};
use std::collections::HashMap;
use std::pin::Pin;

pub(crate) enum ServiceMessage<Msg> {
    Update(Msg),
    Tx(Id, TxServiceMessage),
    Stopped(),
}

pub(crate) struct ServiceCollection<Msg> {
    services: HashMap<Id, ServiceControl>,
    msg_receiver: UnboundedReceiver<ServiceMessage<Msg>>,
    msg_sender: UnboundedSender<ServiceMessage<Msg>>,
}

impl<Msg> Stream for ServiceCollection<Msg> {
    type Item = ServiceMessage<Msg>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.msg_receiver).poll_next(cx)
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

    fn stop_all(mut self) {
        self.services.drain().for_each(|x| x.1.stop());
    }

    pub(crate) fn send(&mut self, id: Id, msg: RxServiceMessage) {
        if let Some(x) = self.services.get(&id) {
            if x.mailbox_tx.unbounded_send(msg).is_err() {
                // the service has terminated
                self.services.remove(&id);
            }
        }
    }
}

struct ServiceControl {
    mailbox_tx: UnboundedSender<RxServiceMessage>,
}

impl ServiceControl {
    fn stop(&self) {
        let _ = self.mailbox_tx.unbounded_send(RxServiceMessage::Stop);
    }
}

pub struct ServiceRunner<Msg: 'static + Send> {
    tx: UnboundedSender<ServiceMessage<Msg>>,
    service: ServiceSubscription<Msg>,
}

impl<Msg: Send> ServiceRunner<Msg> {
    pub(crate) fn run(self) {
        let runner = self;
        crate::platform::spawn(async {
            let id = runner.service.id();
            let mut service = runner.service;
            let mut txmailbox_rx = service.txmailbox_rx.take().unwrap();
            loop {
                select! {
                    tx_msg = txmailbox_rx.next().fuse() => {
                        if let Some(tx_msg) = tx_msg {
                            if runner.tx.unbounded_send(ServiceMessage::Tx(id, tx_msg)).is_err() {
                                break
                            }
                        } else {
                            break
                        }
                    },
                    next_value = service.next().fuse() => {
                        if let Some(x) = next_value {
                            if runner.tx.unbounded_send(ServiceMessage::Update(x)).is_err() {
                                break
                            }
                        } else {
                            break
                        }
                    },
                };
            }
        });
    }
}

