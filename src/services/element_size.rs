use crate::service::{SimpleService, ServiceMailbox, SimpleServiceContainer, RxServiceMessage};
use futures::StreamExt;
use futures::channel::mpsc::UnboundedSender;
use async_std::task;
use serde::{Serialize,Deserialize};
use handlebars::Handlebars;

pub struct ElementSizeNotifier {
    html_id: String,
}

const JS: &'static str = r#"
(function(ctx) {
    var elem = null;
    let size = [-1,-1];
    setInterval(function() {
        if (elem === null) {
            elem = document.getElementById("{{element_id}}");
        }
        if (elem === null) { return; }
        var new_dx = elem.offsetWidth;
        var new_dy = elem.offsetHeight;
        if (size[0] != new_dx || size[1] != new_dy) {
            size[0] = new_dx;
            size[1] = new_dy;
            let data = JSON.stringify([new_dx, new_dy]);
            ctx.send(data);
        }
    }, 50);
})(ctx);
"#;

#[derive(Serialize, Deserialize)]
struct TemplateData {
    element_id: String,
}

impl SimpleService for ElementSizeNotifier {
    type Data = (i32,i32);

    fn run(mut self, mut mailbox: ServiceMailbox, sender: UnboundedSender<Self::Data>) {
        let handlebars = Handlebars::new();
        let data = TemplateData {
            element_id: self.html_id.clone(),
        };
        let js = handlebars.render_template(JS, &data).unwrap();
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
    pub fn create<S: Into<String>>(html_id: S) -> SimpleServiceContainer<ElementSizeNotifier> {
        SimpleServiceContainer::new(Self {
            html_id: html_id.into()
        })
    }

    fn process_msg(&mut self, msg: RxServiceMessage) -> Option<(i32,i32)> {
        let data = match msg {
            RxServiceMessage::Frontend(x) => x,
        };
        serde_json::from_str(&data).ok()
    }

}
