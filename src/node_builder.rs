use crate::blob::Blob;
use crate::dom::DomEvent;
use crate::element::Element;
use crate::event::Subscription;
use crate::listener::{Listener, Rpc};
use crate::node::{Node, NodeItems};
use crate::vdom::Attr;
use crate::Id;
use serde_json::Value as JsonValue;
use std::iter::{once, Once};
use std::marker::PhantomData;
use std::option;
use std::sync::{Arc, Mutex};
use std::vec;

pub struct NodeBuilder<T> {
    namespace: Option<String>,
    marker: PhantomData<T>,
}

impl<T: 'static + Send> NodeBuilder<T> {
    pub fn new() -> NodeBuilder<T> {
        NodeBuilder {
            namespace: None,
            marker: PhantomData,
        }
    }

    pub fn new_with_ns<S: Into<String>>(namespace: S) -> NodeBuilder<T> {
        NodeBuilder {
            namespace: Some(namespace.into()),
            marker: PhantomData,
        }
    }

    pub fn elem<S>(&self, name: S) -> ElementBuilder<T>
    where
        S: Into<String>,
    {
        ElementBuilder::new(name.into(), self.namespace.clone())
    }

    pub fn text<S: Into<String>>(&self, text: S) -> Node<T> {
        Node(NodeItems::Text(text.into()))
    }

    pub fn blob(&self, hash: u64) -> BlobBuilder {
        BlobBuilder {
            id: None,
            hash,
            mime_type: "".to_string(),
            data: vec![],
            on_change: None,
            on_add: None,
        }
    }
}

impl<T: 'static + Send> Default for NodeBuilder<T> {
    fn default() -> Self {
        NodeBuilder::new()
    }
}

pub struct BlobBuilder {
    pub(crate) id: Option<Id>,
    pub(crate) hash: u64,
    pub(crate) mime_type: String,
    pub(crate) data: Vec<u8>,
    pub(crate) on_change: Option<String>,
    pub(crate) on_add: Option<String>,
}

impl BlobBuilder {
    pub fn mime_type<S: Into<String>>(mut self, mime_type: S) -> Self {
        self.mime_type = mime_type.into();
        self
    }

    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    pub fn on_change<T: Into<String>>(mut self, js: T) -> Self {
        self.on_change = Some(js.into());
        self
    }

    pub fn on_add<T: Into<String>>(mut self, js: T) -> Self {
        self.on_add = Some(js.into());
        self
    }

    pub fn build(self) -> Blob {
        self.into()
    }
}

pub struct ElementBuilder<T: 'static + Send> {
    pub(crate) id: Id,
    pub(crate) tag: String,
    pub(crate) attrs: Vec<Attr>,
    pub(crate) js_events: Vec<Attr>,
    pub(crate) listeners: Vec<Listener<T>>,
    pub(crate) children: Vec<Node<T>>,
    pub(crate) namespace: Option<String>,
    pub(crate) classes: Vec<String>,
    pub(crate) html_id: Option<String>,
    pub(crate) rpc: Option<Rpc<T>>,
}

impl<T: 'static + Send> ElementBuilder<T> {
    fn new(tag: String, namespace: Option<String>) -> ElementBuilder<T> {
        ElementBuilder {
            id: Id::new_empty(),
            tag,
            attrs: Vec::new(),
            js_events: Vec::new(),
            listeners: Vec::new(),
            children: Vec::new(),
            namespace,
            classes: vec![],
            html_id: None,
            rpc: None,
        }
    }

    pub fn on<S: Into<String>, F: 'static + Send + Fn(DomEvent) -> T>(
        mut self,
        name: S,
        fun: F,
    ) -> Self {
        if self.id.is_empty() {
            self.id = Id::new();
        }
        self.listeners.push(Listener {
            event_name: name.into(),
            node_id: self.id,
            fun: Arc::new(Mutex::new(Box::new(fun))),
            no_propagate: false,
            prevent_default: false,
        });
        self
    }

    pub fn listener<S: Into<String>, F: 'static + Send + Fn(DomEvent) -> T>(
        self,
        name: S,
        fun: F,
    ) -> ListenerBuilder<T> {
        ListenerBuilder {
            parent: self,
            name: name.into(),
            fun: Arc::new(Mutex::new(fun)),
            prevent_default: false,
            no_propagate: false,
        }
    }

    pub fn rpc<F>(mut self, fun: F) -> Self
    where
        F: 'static + Send + Fn(JsonValue) -> T,
    {
        if self.id.is_empty() {
            self.id = Id::new();
        }
        let rpc = Rpc {
            node_id: self.id,
            fun: Arc::new(Mutex::new(fun)),
        };
        self.rpc = Some(rpc);
        self
    }

    pub fn attr<R: ToString, S: ToString>(mut self, key: R, value: S) -> Self {
        self.attrs.push(Attr {
            key: key.to_string(),
            value: value.to_string(),
        });
        self
    }

    /// Register an javascript event handler which listens to the event given
    /// by `key` and executes the function defined in `value`. The function receives a
    /// single $event argument pointing to the [event](https://developer.mozilla.org/en-US/docs/Web/API/Event):
    ///
    /// ```
    /// # use greenhorn::node::Node;
    /// fn render() -> Node<()> {
    ///     Node::html()
    ///         .elem("div")
    ///         .js_event("click", "console.log($event.button)")
    ///         .build()
    /// }
    /// ```
    pub fn js_event<R: ToString, S: ToString>(mut self, key: R, value: S) -> Self {
        self.js_events.push(Attr {
            key: key.to_string(),
            value: value.to_string(),
        });
        self
    }

    pub fn add<U, V>(mut self, children: V) -> Self
    where
        U: Iterator<Item = Node<T>>,
        V: Into<NodeIter<T, U>>,
    {
        for child in children.into() {
            self.children.push(child);
        }
        self
    }

    pub fn text<S: Into<String>>(mut self, text: S) -> Self {
        self.children.push(Node(NodeItems::Text(text.into())));
        self
    }

    pub fn id<S: Into<String>>(mut self, id: S) -> Self {
        self.html_id = Some(id.into());
        self
    }

    pub fn class<S: Into<String>>(mut self, class: S) -> Self {
        self.classes.push(class.into());
        self
    }

    pub fn build(mut self) -> Node<T> {
        if !self.classes.is_empty() {
            let cls = self.classes.join(" ");
            self.attrs.push(Attr {
                key: "class".to_string(),
                value: cls,
            });
        }
        if let Some(x) = self.html_id.take() {
            self.attrs.push(Attr {
                key: "id".to_string(),
                value: x,
            })
        }

        Node(NodeItems::Element(Element {
            id: self.id,
            tag: Some(self.tag),
            attrs: Some(self.attrs),
            js_events: Some(self.js_events),
            listeners: Some(self.listeners),
            children: Some(self.children),
            namespace: self.namespace,
            rpc: self.rpc,
        }))
    }
}

pub struct ListenerBuilder<T: 'static + Send> {
    parent: ElementBuilder<T>,
    name: String,
    fun: Arc<Mutex<dyn Send + Fn(DomEvent) -> T>>,
    prevent_default: bool,
    no_propagate: bool,
}

impl<T: 'static + Send> ListenerBuilder<T> {
    pub fn prevent_default(mut self) -> Self {
        self.prevent_default = true;
        self
    }

    pub fn no_propagate(mut self) -> Self {
        self.no_propagate = true;
        self
    }

    pub fn build(mut self) -> ElementBuilder<T> {
        if self.parent.id.is_empty() {
            self.parent.id = Id::new();
        }
        self.parent.listeners.push(Listener {
            event_name: self.name,
            node_id: self.parent.id,
            fun: self.fun.clone(),
            no_propagate: self.no_propagate,
            prevent_default: self.prevent_default,
        });
        self.parent
    }
}

impl<T: 'static + Send> From<ElementBuilder<T>> for Node<T> {
    fn from(builder: ElementBuilder<T>) -> Self {
        builder.build()
    }
}

impl<T: 'static + Send> From<String> for Node<T> {
    fn from(value: String) -> Self {
        Node(NodeItems::Text(value))
    }
}

impl<T: 'static + Send> From<&str> for Node<T> {
    fn from(value: &str) -> Self {
        Node(NodeItems::Text(value.into()))
    }
}

impl<T: 'static + Send> From<Subscription<T>> for Node<T> {
    fn from(value: Subscription<T>) -> Self {
        Node(NodeItems::EventSubscription(value.id(), value))
    }
}

pub struct NodeIter<T: 'static + Send, U: Iterator<Item = Node<T>>> {
    pub(crate) inner: U,
}

impl<T: 'static + Send, U: Iterator<Item = Node<T>>> Iterator for NodeIter<T, U> {
    type Item = Node<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<T: 'static + Send> From<Option<Node<T>>> for NodeIter<T, option::IntoIter<Node<T>>> {
    fn from(value: Option<Node<T>>) -> Self {
        NodeIter {
            inner: value.into_iter(),
        }
    }
}

impl<T: 'static + Send> From<Blob> for NodeIter<T, Once<Node<T>>> {
    fn from(value: Blob) -> Self {
        NodeIter {
            inner: once(Node(NodeItems::Blob(value))),
        }
    }
}

impl<T: 'static + Send, U: Iterator<Item = Node<T>>> From<U> for NodeIter<T, U> {
    fn from(value: U) -> Self {
        NodeIter { inner: value }
    }
}

impl<T: 'static + Send> From<Vec<Node<T>>> for NodeIter<T, vec::IntoIter<Node<T>>> {
    fn from(value: Vec<Node<T>>) -> Self {
        NodeIter {
            inner: value.into_iter(),
        }
    }
}

impl<T: 'static + Send> From<&str> for NodeIter<T, Once<Node<T>>> {
    fn from(value: &str) -> Self {
        NodeIter {
            inner: once(Node(NodeItems::Text(value.to_string()))),
        }
    }
}

impl<T: 'static + Send> From<String> for NodeIter<T, Once<Node<T>>> {
    fn from(value: String) -> Self {
        NodeIter {
            inner: once(Node(NodeItems::Text(value))),
        }
    }
}

impl<T: 'static + Send> From<Subscription<T>> for NodeIter<T, Once<Node<T>>> {
    fn from(value: Subscription<T>) -> Self {
        NodeIter {
            inner: once(value.into()),
        }
    }
}

impl<T: 'static + Send> From<Node<T>> for NodeIter<T, Once<Node<T>>> {
    fn from(value: Node<T>) -> Self {
        NodeIter { inner: once(value) }
    }
}

impl<T: 'static + Send> From<ElementBuilder<T>> for NodeIter<T, Once<Node<T>>> {
    fn from(value: ElementBuilder<T>) -> Self {
        NodeIter {
            inner: once(value.build()),
        }
    }
}

impl<T: 'static + Send> From<&Blob> for NodeIter<T, Once<Node<T>>> {
    fn from(value: &Blob) -> Self {
        NodeIter {
            inner: once(Node(NodeItems::Blob(Blob {
                inner: value.inner.clone(),
            }))),
        }
    }
}

impl<T: 'static + Send> From<&String> for NodeIter<T, Once<Node<T>>> {
    fn from(value: &String) -> Self {
        NodeIter {
            inner: once(Node(NodeItems::Text(value.clone()))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::{BaseEvent, InputValue};
    use crate::Render;
    use assert_matches::assert_matches;

    #[derive(Debug)]
    enum Msg {
        None,
        Clicked,
    }

    fn builder() -> NodeBuilder<Msg> {
        NodeBuilder::<Msg>::new()
    }

    #[test]
    fn test_builder() {
        let elem = builder()
            .elem("div")
            .attr("class", "foo")
            .attr("foo", "bar")
            .on("click", |_| Msg::Clicked)
            .build();

        if let NodeItems::Element(e) = elem.0 {
            assert_eq!(e.tag.unwrap(), "div");
            let listeners = &e.listeners.unwrap();
            let listener = listeners.get(0).unwrap();
            assert_eq!(listener.event_name, "click");
            let evt = BaseEvent {
                target: Default::default(),
                event_name: "".to_string(),
                target_value: InputValue::NoValue,
            };
            let msg = (listener.fun.lock().unwrap())(DomEvent::Base(evt));
            assert_matches!(msg, Msg::Clicked);
        } else {
            panic!()
        }
    }

    struct RenderImpl {}

    impl Render for RenderImpl {
        type Message = ();

        fn render(&self) -> Node<Self::Message> {
            Node::svg().elem("svg").build()
        }
    }

    #[test]
    fn test_namespace() {
        let render = RenderImpl {};
        let node = render.render();
        if let NodeItems::Element(e) = node.0 {
            assert_eq!(e.namespace, Some("http://www.w3.org/2000/svg".into()))
        } else {
            panic!()
        }
    }

    #[test]
    fn test_listener() {
        let node = builder()
            .elem("div")
            .attr("test", "foo")
            .listener("click", |_| Msg::Clicked)
            .prevent_default()
            .no_propagate()
            .build()
            .attr("foo", "bar")
            .build();
        if let NodeItems::Element(elem) = node.0 {
            let attrs = elem.attrs.as_ref().unwrap();
            assert_eq!(attrs.len(), 2);
            assert_eq!(attrs[0].key, "test");
            assert_eq!(attrs[0].value, "foo");
            assert_eq!(attrs[1].key, "foo");
            assert_eq!(attrs[1].value, "bar");
            let listeners = elem.listeners.as_ref().unwrap();
            assert_eq!(listeners.len(), 1);
            assert_eq!(listeners[0].event_name, "click");
            assert!(listeners[0].no_propagate);
            assert!(listeners[0].prevent_default);
            assert_eq!(listeners[0].node_id, elem.id);
        } else {
            panic!()
        }
    }

    #[test]
    fn test_children() {
        let node = builder().elem("div").add(builder().elem("pre")).build();
        if let NodeItems::Element(elem) = node.0 {
            assert_eq!(elem.tag.unwrap(), "div");
            assert_eq!(elem.children.as_ref().unwrap().len(), 1);
            let child_node = &elem.children.as_ref().unwrap()[0];
            if let NodeItems::Element(elem) = &child_node.0 {
                assert_eq!(elem.tag.as_ref().unwrap(), "pre");
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }
}
