use futures::channel::mpsc::{UnboundedSender, UnboundedReceiver};
use crate::service::{ServiceSubscription, ServiceMailbox};
use async_std::task;
use futures::{StreamExt, FutureExt};
use futures::select;

enum ServiceControl {
    Cancel,
}

struct ServiceRunner<Msg: 'static> {
    msg_sender: UnboundedSender<Msg>,
    control_rx: UnboundedReceiver<ServiceControl>,
    service: ServiceSubscription<Msg>,
    mailbox: ServiceMailbox,
}

impl<Msg: Send> ServiceRunner<Msg> {
    fn new(
        service: ServiceSubscription<Msg>,
        msg_sender: UnboundedSender<Msg>,
        control_rx: UnboundedReceiver<ServiceControl>,
        mailbox: ServiceMailbox
    ) -> ServiceRunner<Msg> {
        ServiceRunner {
            msg_sender,
            control_rx,
            service,
            mailbox
        }
    }

    fn run(self) -> task::JoinHandle<()> {
        let mut runner = self;
        task::spawn(async {
            runner.service.setup(runner.mailbox);
            loop {
                select! {
                    next_value = runner.service.next().fuse() => {
                        if let Some(x) = next_value {
                            if let Err(_) = runner.msg_sender.unbounded_send(x) {
                                runner.service.stop();
                                break
                            }
                        } else {
                            runner.service.stop();
                            break
                        }
                    },
                    _control = runner.control_rx.next().fuse() => {
                        runner.service.stop();
                        break
                    }
                }
            }
        })
    }
}