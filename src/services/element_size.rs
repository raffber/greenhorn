use crate::service::{SimpleService, ServiceMailbox, SimpleServiceContainer, RxServiceMessage};
use futures::StreamExt;
use futures::channel::mpsc::UnboundedSender;
use async_std::task;

struct ElementSizeNotifier {
    html_id: String,
}

const JS: &'static str = r#"
function(ctx) {{
    var elem = null;
    setInterval(function() {{
        if (elem === null) {{
            elem = document.getElementById("{}");
        }}
        if (elem === null) {{ return; }}
        var json = JSON.stringify({{
            dx: elem.offsetWidth(),
            dy: elem.offsetHeight()
        }});
        ctx.send(json);
    }}, 50);
}}
"#;

enum ElementSizeMsg {
    Changed{dx: i32, dy: i32}
}

impl SimpleService for ElementSizeNotifier {
    type Data = ElementSizeMsg;

    fn run(mut self, mut mailbox: ServiceMailbox, sender: UnboundedSender<Self::Data>) {
        let js = format!(JS, self.html_id);
        mailbox.run_js(js);
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

impl ElementSizeNotifier {
    fn create<S: Into<String>>(html_id: S) -> SimpleServiceContainer<ElementSizeNotifier> {
        SimpleServiceContainer::new(Self {
            html_id: html_id.into()
        })
    }

    fn process_msg(&mut self, _msg: RxServiceMessage) -> Option<ElementSizeMsg> {
        None
    }

}
