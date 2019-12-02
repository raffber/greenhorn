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
    let size = [-1, -1, -1,-1];
    setInterval(function() {
        if (elem === null) {
            elem = document.getElementById("{{element_id}}");
        }
        if (elem === null) { return; }
        var dx = elem.offsetWidth;
        var dy = elem.offsetHeight;
        var rect = elem.getBoundingClientRect();
        var x = rect.left;
        var y = rect.right;
        if (size[0] != x || size[1] != y || size[2] != dx || size[3] != dy) {
            size[0] = x;
            size[1] = y;
            size[2] = dx;
            size[3] = dy;
            var data = JSON.stringify({
                "x": x,
                "y": y,
                "dx": dx,
                "dy": dy
            });
            ctx.send(data);
        }
    }, 50);
})(ctx);
"#;

#[derive(Serialize, Deserialize)]
struct TemplateData {
    element_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ElementSize {
    pub x: i32,
    pub y: i32,
    pub dx: i32,
    pub dy: i32,
}

impl SimpleService for ElementSizeNotifier {
    type Data = ElementSize;

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

    fn process_msg(&mut self, msg: RxServiceMessage) -> Option<ElementSize> {
        let data = match msg {
            RxServiceMessage::Frontend(x) => x,
        };
        serde_json::from_str(&data).ok()
    }

}
