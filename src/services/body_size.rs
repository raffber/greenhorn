use crate::service::{SimpleService, ServiceMailbox, SimpleServiceContainer, RxServiceMessage};
use futures::StreamExt;
use futures::channel::mpsc::UnboundedSender;
use async_std::task;

struct BodySizeNotifier;

const JS: &'static str = "

";

enum BodySizeMsg {
    Changed{dx: i32, dy: i32}
}

impl SimpleService for BodySizeNotifier {
    type Data = BodySizeMsg;

    fn run(mut self, mut mailbox: ServiceMailbox, sender: UnboundedSender<Self::Data>) {
        mailbox.run_js(JS);
        task::spawn(async move {
            loop {
                let msg = mailbox.next().await;
                if let Some(msg) = msg {
                    if let Some(x) = self.process_msg(msg) {
                        let _ = sender.unbounded_send(x);
                    }
                } else {
                    break;
                }
            }
        });
    }
}

impl BodySizeNotifier {
    fn create() -> SimpleServiceContainer<BodySizeNotifier> {
        SimpleServiceContainer::new(Self {})
    }

    fn process_msg(&mut self, _msg: RxServiceMessage) -> Option<BodySizeMsg> {
        None
    }

}
