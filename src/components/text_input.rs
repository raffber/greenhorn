use crate::context::Context;
use crate::dom::DomEvent;
use crate::event::Event;
use crate::node::Node;
use crate::node_builder::{ElementBuilder, NodeIter};
use crate::vdom::Attr;
use std::collections::HashMap;
use std::iter::{once, Once};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct SubscribedEvent {
    component_event: Event<DomEvent>,
    evt: DomEvent,
}

#[derive(Debug)]
pub enum TextInputMsg {
    ValueChange(DomEvent),
    SubscribedEvent(SubscribedEvent),
}

pub struct TextInputSubscription<T: 'static + Send> {
    event: Event<DomEvent>,
    mapper: Arc<Mutex<dyn 'static + Send + Fn(DomEvent) -> T>>,
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

    pub fn update<T: 'static + Send>(&mut self, msg: TextInputMsg, ctx: &Context<T>) {
        match msg {
            TextInputMsg::ValueChange(evt) => self.text = evt.target_value().get_text().unwrap(),
            TextInputMsg::SubscribedEvent(subs) => {
                ctx.emit(&subs.component_event, subs.evt);
            }
        }
    }

    pub fn render<T: 'static + Send, F: 'static + Send + Fn(TextInputMsg) -> T>(
        &self,
        mapper: F,
    ) -> TextInputRender<T> {
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
        let input_node = Node::html()
            .elem("input")
            .attr("type", "text")
            .attr("__value_version", self.version)
            .attr("value", &self.text)
            .on("change", TextInputMsg::ValueChange)
            .js_event("render", render_fun);
        TextInputRender {
            mapper: Arc::new(Mutex::new(mapper)),
            input_node,
            events: Default::default(),
        }
    }
}

pub struct TextInputRender<T: 'static + Send> {
    mapper: Arc<Mutex<dyn 'static + Send + Fn(TextInputMsg) -> T>>,
    input_node: ElementBuilder<TextInputMsg>,
    events: HashMap<String, TextInputSubscription<T>>,
}

impl<T: 'static + Send> TextInputRender<T> {
    pub fn on<S: Into<String>, F: 'static + Send + Fn(DomEvent) -> T>(
        mut self,
        evt_name: S,
        mapper: F,
    ) -> Self {
        let subs = TextInputSubscription {
            event: Default::default(),
            mapper: Arc::new(Mutex::new(mapper)),
        };
        self.events.insert(evt_name.into(), subs);
        self
    }

    pub fn id<S: Into<String>>(mut self, id: S) -> Self {
        self.input_node.html_id = Some(id.into());
        self
    }

    pub fn class<S: Into<String>>(mut self, class: S) -> Self {
        self.input_node.classes.push(class.into());
        self
    }

    pub fn attr<R: ToString, S: ToString>(mut self, key: R, value: S) -> Self {
        self.input_node.attrs.push(Attr {
            key: key.to_string(),
            value: value.to_string(),
        });
        self
    }

    pub fn render(self) -> Node<T> {
        let mut parent = Node::html().elem("div");
        let mut input_node = self.input_node;
        for (evt_name, subs) in &self.events {
            let event_cloned = subs.event.clone();
            let fun = move |evt| {
                let data = SubscribedEvent {
                    component_event: event_cloned.clone(),
                    evt,
                };
                TextInputMsg::SubscribedEvent(data)
            };
            input_node = input_node.on(evt_name, fun);
            let fun = subs.mapper.clone();
            let subscription = subs.event.subscribe(move |evt| (*fun.lock().unwrap())(evt));
            parent = parent.add(subscription);
        }
        parent = parent.add(input_node.build().map_shared(self.mapper));
        parent.build()
    }
}

impl<T: 'static + Send> From<TextInputRender<T>> for Node<T> {
    fn from(value: TextInputRender<T>) -> Self {
        value.render()
    }
}

impl<T: 'static + Send> From<TextInputRender<T>> for NodeIter<T, Once<Node<T>>> {
    fn from(value: TextInputRender<T>) -> Self {
        NodeIter {
            inner: once(value.render()),
        }
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}
