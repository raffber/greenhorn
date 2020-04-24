use crate::service::{Mailbox, RxServiceMessage, Service};
use futures::StreamExt;
use futures::channel::mpsc::{UnboundedReceiver, unbounded};
use serde::{Serialize,Deserialize};
use handlebars::Handlebars;

pub struct ElementSizeNotifier {
    html_id: String,
}

const JS: &str = r#"
(function(ctx) {
    let size = [-1, -1, -1,-1];
    setInterval(function() {
        var elem = document.getElementById("{{element_id}}");
        var dx = elem.offsetWidth;
        var dy = elem.offsetHeight;
        var rect = elem.getBoundingClientRect();
        var x = rect.left;
        var y = rect.top;
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

#[derive(Serialize, Deserialize, Debug)]
pub struct ElementSize {
    pub x: i32,
    pub y: i32,
    pub dx: i32,
    pub dy: i32,
}

impl Service for ElementSizeNotifier {
    type Data = ElementSize;
    type DataStream = UnboundedReceiver<ElementSize>;

    fn start(mut self, mut mailbox: Mailbox) -> Self::DataStream {
        let handlebars = Handlebars::new();
        let data = TemplateData {
            element_id: self.html_id.clone(),
        };
        let js = handlebars.render_template(JS, &data).unwrap();
        let (tx, rx) = unbounded();
        mailbox.run_js(js);
        crate::platform::spawn(async move {
            loop {
                let msg = mailbox.next().await;
                if let Some(msg) = msg {
                    if let Some(x) = self.process_msg(msg) {
                        let _ = tx.unbounded_send(x);
                    } else {
                        break
                    }
                } else {
                    break;
                }
            }
        });
        rx
    }
}

impl ElementSizeNotifier {
    pub fn new<S: Into<String>>(html_id: S) -> Self {
        Self {
            html_id: html_id.into()
        }
    }

    fn process_msg(&mut self, msg: RxServiceMessage) -> Option<ElementSize> {
        match msg {
            RxServiceMessage::Frontend(data) => {
                serde_json::from_str(&data).unwrap()
            },
            RxServiceMessage::Stop => {
                None
            },
        }
    }

}
