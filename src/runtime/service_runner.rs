use crate::service::{RxServiceMessage, ServiceMailbox, ServiceSubscription, TxServiceMessage};
use crate::Id;
use async_std::task;
use async_std::task::JoinHandle;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::select;
use futures::task::{Context, Poll};
use futures::{FutureExt, Stream, StreamExt};
use std::collections::HashMap;
use std::pin::Pin;

enum ServiceControlMsg {
    Stop,
}

pub enum ServiceMessage<Msg> {
    Update(Msg),
    Tx(Id, TxServiceMessage),
    Stopped(),
}

pub struct ServiceCollection<Msg> {
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
        let (tx, rx) = unbounded();
        let (txmailbox_tx, txmailbox_rx) = unbounded::<TxServiceMessage>();
        let (rxmailbox_tx, rxmailbox_rx) = unbounded::<RxServiceMessage>();
        let id = subs.id();
        let mailbox = ServiceMailbox {
            rx: rxmailbox_rx,
            tx: txmailbox_tx,
        };

        let runner = ServiceRunner {
            tx: self.msg_sender.clone(),
            rx,
            service: subs,
            mailbox_rx: txmailbox_rx,
            mailbox,
        };

        let control = ServiceControl {
            tx,
            handle: runner.run(),
            mailbox_tx: rxmailbox_tx,
        };

        self.services.insert(id, control);
    }

    fn stop_all(mut self) {
        self.services.drain().for_each(|x| x.1.stop());
    }

    fn send(&mut self, id: Id, msg: RxServiceMessage) {
        if let Some(x) = self.services.get(&id) {
            if x.mailbox_tx.unbounded_send(msg).is_err() {
                // the service has terminated
                self.services.remove(&id);
            }
        }
    }
}

struct ServiceControl {
    tx: UnboundedSender<ServiceControlMsg>,
    handle: JoinHandle<()>,
    mailbox_tx: UnboundedSender<RxServiceMessage>,
}

impl ServiceControl {
    #[inline]
    fn stop(self) {
        let _ = self.tx.unbounded_send(ServiceControlMsg::Stop);
    }
}

pub struct ServiceRunner<Msg: 'static> {
    tx: UnboundedSender<ServiceMessage<Msg>>,
    rx: UnboundedReceiver<ServiceControlMsg>,
    service: ServiceSubscription<Msg>,
    mailbox_rx: UnboundedReceiver<TxServiceMessage>,
    mailbox: ServiceMailbox,
}

impl<Msg: Send> ServiceRunner<Msg> {
    pub(crate) fn run(self) -> task::JoinHandle<()> {
        let mut runner = self;
        task::spawn(async {
            runner.service.setup(runner.mailbox);
            let id = runner.service.id();
            loop {
                select! {
                    tx_msg = runner.mailbox_rx.next().fuse() => {
                        if let Some(tx_msg) = tx_msg {
                            if runner.tx.unbounded_send(ServiceMessage::Tx(id, tx_msg)).is_err() {
                                runner.service.stop();
                                break
                            }
                        }
                    },
                    next_value = runner.service.next().fuse() => {
                        if let Some(x) = next_value {
                            if runner.tx.unbounded_send(ServiceMessage::Update(x)).is_err() {
                                runner.service.stop();
                                break
                            }
                        } else {
                            runner.service.stop();
                            let _ = runner.tx.unbounded_send(ServiceMessage::Stopped());
                            break
                        }
                    },
                    _control = runner.rx.next().fuse() => {
                        runner.service.stop();
                        break
                    }
                };
            }
        })
    }
}
