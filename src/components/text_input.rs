use crate::node::Node;
use crate::dom::DomEvent;
use std::marker::PhantomData;
use std::collections::HashMap;
use crate::event::Event;
use crate::context::Context;
use std::sync::{Arc, Mutex};

pub enum TextInputMsg {
    ValueChange(DomEvent),
    SubscribedEvent(String, DomEvent),
}

pub struct TextInputSubscription<T: 'static + Send> {
    event: Event<DomEvent>,
    mapper: Arc<Mutex<dyn 'static + Send + Fn(DomEvent) -> T>>,
}

pub struct TextInput<T: 'static + Send> {
    text: String,
    version: u32,
    marker: PhantomData<T>,
    events: HashMap<String, TextInputSubscription<T>>,
}

impl<T: 'static + Send> TextInput<T> {
    pub fn new() -> Self {
        Self {
            text: "".to_string(),
            version: 0,
            marker: PhantomData,
            events: Default::default()
        }
    }

    pub fn set<S: Into<String>>(&mut self, value: S) {
        self.text = value.into();
        self.version += 1;
    }


    pub fn get(&self) -> &str {
        &self.text
    }

    pub fn update(&mut self, msg: TextInputMsg, ctx: &Context<T>) {
        match msg {
            TextInputMsg::ValueChange(evt) => {
                self.text = evt.target_value().get_text().unwrap()
            },
            TextInputMsg::SubscribedEvent(evt_name, evt) => {
                if let Some(comp_evt) = self.events.get(&evt_name) {
                    ctx.emit(&comp_evt.event, evt);
                }
            }
        }
    }

    pub fn subscribe<S: Into<String>, F: 'static + Send + Fn(DomEvent) -> T>(&mut self, evt_name: S, mapper: F) {
        let subs = TextInputSubscription {
            event: Default::default(),
            mapper: Arc::new(Mutex::new(mapper))
        };
        self.events.insert(evt_name.into(), subs);
    }

    pub fn unsubscribe(&mut self, evt_name: &str) {
        self.events.remove(evt_name);
    }

    pub fn render<F: 'static + Send + Fn(TextInputMsg) -> T>(&self, mapper: F) -> Node<T> {
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
        let mut input_node = Node::html().elem("input")
            .attr("type", "text")
            .attr("__value_version", self.version)
            .attr("value", &self.text)
            .on("change", TextInputMsg::ValueChange)
            .js_event("render", render_fun);
        let mut parent = Node::html().elem("div");
        for (evt_name, subs) in &self.events {
            let evt_name_clone = evt_name.clone();
            input_node = input_node.on(evt_name, move |evt| TextInputMsg::SubscribedEvent(evt_name_clone.clone(), evt));
            let fun = subs.mapper.clone();
            let subscription = subs.event.subscribe(move |evt| (*fun.lock().unwrap())(evt));
            parent = parent.add(subscription);
        }
        parent = parent.add(input_node.build().map(mapper));
        parent.build()
    }
}

impl<T: 'static + Send> Default for TextInput<T> {
    fn default() -> Self {
        Self::new()
    }
}
