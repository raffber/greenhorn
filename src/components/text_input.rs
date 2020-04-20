use crate::node::Node;
use crate::dom::DomEvent;
use crate::node_builder::ElementBuilder;

pub enum TextInputMsg {
    ValueChange(DomEvent),
}

pub struct TextInput {
    text: String,
    version: u32,
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            text: "".to_string(),
            version: 0,
        }
    }

    pub fn set<S: Into<String>>(&mut self, value: S) {
        self.text = value.into();
        self.version += 1;
    }


    pub fn get(&self) -> &str {
        &self.text
    }

    pub fn update(&mut self, msg: TextInputMsg) {
        match msg {
            TextInputMsg::ValueChange(evt) => {
                self.text = evt.target_value().get_text().unwrap()
            },
        }
    }

    pub fn render(&self) -> ElementBuilder<TextInputMsg> {
        // we use this very simple technique to move the actual DOM diffing to the frontend side:
        // if we set the new text value, we also bump the version.
        // the version is also recorded in a custom attribute. If the frontend sees that
        // the version has been bumped, it copies the value attribute to the actual target
        // value.
        let render_fun = "{
            let rendered_version = event.target.getAttribute('__rendered_version');
            let value_version = event.target.getAttribute('__value_version');
            if (rendered_version != value_version) {
                event.target.value = event.target.getAttribute('value');
                event.target.setAttribute('__rendered_version', value_version);
            }
        }";
        Node::html().elem("input")
            .attr("type", "text")
            .attr("__value_version", self.version)
            .attr("value", &self.text)
            .on("change", TextInputMsg::ValueChange)
            .js_event("render", render_fun)
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}
