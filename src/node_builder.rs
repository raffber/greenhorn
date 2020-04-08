use crate::component::Component;
use crate::dom_event::DomEvent;
use crate::vdom::Attr;
use crate::{Id, Render};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use crate::node::Node;
use crate::element::Element;
use crate::listener::Listener;
use crate::blob::Blob;
use std::iter::{Once, once};
use crate::event::Subscription;
use std::option;
use std::vec;


pub struct NodeBuilder<T> {
    namespace: Option<String>,
    marker: PhantomData<T>,
}

impl<T: 'static> NodeBuilder<T> {
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
        Node::Text(text.into())
    }

    pub fn blob(&self, hash: u64) -> BlobBuilder {
        BlobBuilder {
            id: None,
            hash,
            mime_type: "".to_string(),
            data: vec![],
            on_change: None,
            on_add: None
        }
    }

    pub fn mount<ChildMsg, R, Mapper>(&self, comp: &Component<R>, mapper: Mapper) -> Node<T>
    where
        ChildMsg: 'static + Send,
        R: 'static + Render<Message = ChildMsg>,
        Mapper: 'static + Send + Fn(ChildMsg) -> T,
    {
        comp.render().map(mapper)
    }
}

impl<T: 'static> Default for NodeBuilder<T> {
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

pub struct ElementBuilder<T: 'static> {
    id: Id,
    tag: String,
    attrs: Vec<Attr>,
    js_events: Vec<Attr>,
    listeners: Vec<Listener<T>>,
    children: Vec<Node<T>>,
    namespace: Option<String>,
    classes: Vec<String>,
    html_id: Option<String>,
}

impl<T: 'static> ElementBuilder<T> {
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
            html_id: None
        }
    }

    pub fn on<S: Into<String>, F: 'static + Send + Fn(DomEvent) -> T>(mut self, name: S, fun: F) -> Self {
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
            fun: Arc::new(Mutex::new(Box::new(fun))),
            prevent_default: false,
            no_propagate: false,
        }
    }

    pub fn attr<R: ToString, S: ToString>(mut self, key: R, value: S) -> Self {
        self.attrs.push(Attr {
            key: key.to_string(),
            value: value.to_string(),
        });
        self
    }

    pub fn js_event<R: ToString, S: ToString>(mut self, key: R, value: S) -> Self {
        self.js_events.push(Attr {
            key: key.to_string(),
            value: value.to_string(),
        });
        self
    }

    pub fn mount<ChildMsg, R, Mapper>(mut self, comp: &Component<R>, mapper: Mapper) -> Self
        where
            ChildMsg: 'static + Send,
            R: 'static + Render<Message = ChildMsg>,
            Mapper: 'static + Send + Fn(ChildMsg) -> T,
    {
        self.children.push(comp.render().map(mapper));
        self
    }

    pub fn add<U, V>(mut self, children: V) -> Self
        where
            U: Iterator<Item=Node<T>>,
            V: Into<NodeIter<T,U>>,
    {
        for child in children.into() {
            self.children.push(child);
        }
        self
    }

    pub fn text<S: Into<String>>(mut self, text: S) -> Self {
        self.children.push(Node::Text(text.into()));
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
            self.attrs.push(Attr { key: "class".to_string(), value: cls });
        }
        if let Some(x) = self.html_id.take() {
            self.attrs.push(Attr { key: "id".to_string(), value: x })
        }

        Node::Element(Element {
            id: self.id,
            tag: Some(self.tag),
            attrs: Some(self.attrs),
            js_events: Some(self.js_events),
            listeners: Some(self.listeners),
            children: Some(self.children),
            namespace: self.namespace,
        })
    }
}

pub struct ListenerBuilder<T: 'static> {
    parent: ElementBuilder<T>,
    name: String,
    fun: Arc<Mutex<Box<dyn Send + Fn(DomEvent) -> T>>>,
    prevent_default: bool,
    no_propagate: bool,
}

impl<T: 'static> ListenerBuilder<T> {
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


impl<T: 'static> From<ElementBuilder<T>> for Node<T> {
    fn from(builder: ElementBuilder<T>) -> Self {
        builder.build()
    }
}

impl<T: 'static> From<String> for Node<T> {
    fn from(value: String) -> Self {
        Node::Text(value)
    }
}

impl<T: 'static> From<&str> for Node<T> {
    fn from(value: &str) -> Self {
        Node::Text(value.into())
    }
}

impl<T: 'static> From<Subscription<T>> for Node<T> {
    fn from(value: Subscription<T>) -> Self {
        Node::EventSubscription(value.id(), value)
    }
}


pub struct NodeIter<T: 'static, U: Iterator<Item=Node<T>>> {
    inner: U,
}

impl<T: 'static, U: Iterator<Item=Node<T>>> Iterator for NodeIter<T, U> {
    type Item = Node<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<T: 'static> From<Option<Node<T>>> for NodeIter<T, option::IntoIter<Node<T>>> {
    fn from(value: Option<Node<T>>) -> Self {
        NodeIter { inner: value.into_iter() }
    }
}

impl<T: 'static> From<Blob> for NodeIter<T, once<Node<T>>> {
    fn from(value: Blob) -> Self {
        NodeIter { inner: once(Node::Blob(value)) }
    }
}

impl<T: 'static> From<Vec<Node<T>>> for NodeIter<T, vec::IntoIter<Node<T>>> {
    fn from(value: Vec<Node<T>>) -> Self {
        NodeIter { inner: value.into_iter() }
    }
}

impl<T: 'static> From<&str> for NodeIter<T, Once<Node<T>>> {
    fn from(value: &str) -> Self {
        NodeIter { inner: once(Node::Text(value.to_string())) }
    }
}

impl<T: 'static> From<String> for NodeIter<T, Once<Node<T>>> {
    fn from(value: String) -> Self {
        NodeIter { inner: once(Node::text(value)) }
    }
}

impl<T: 'static> From<Subscription<T>> for NodeIter<T, Once<Node<T>>> {
    fn from(value: Subscription<T>) -> Self {
        NodeIter { inner: once(value.into())}
    }
}

impl<T: 'static> From<Node<T>> for NodeIter<T, Once<Node<T>>> {
    fn from(value: Node<T>) -> Self {
        NodeIter { inner: once(value) }
    }
}

impl<T: 'static> From<ElementBuilder<T>> for NodeIter<T, Once<Node<T>>> {
    fn from(value: ElementBuilder<T>) -> Self {
        NodeIter { inner: once(value.build()) }
    }
}


impl<T: 'static> From<&Blob> for NodeIter<T, Once<Node<T>>> {
    fn from(value: &Blob) -> Self {
        NodeIter { inner: once(Node::Blob(Blob {
            inner: value.inner.clone()
        })) }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
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

        if let Node::Element(e) = elem {
            assert_eq!(e.tag.unwrap(), "div");
            let listeners = &e.listeners.unwrap();
            let listener = listeners.get(0).unwrap();
            assert_eq!(listener.event_name, "click");
            let msg = (listener.fun.lock().unwrap())(DomEvent::Base(Id::new(), "".into()));
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
        if let Node::Element(e) = node {
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
        if let Node::Element(elem) = node {
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
        if let Node::Element(elem) = node {
            assert_eq!(elem.tag.unwrap(), "div");
            assert_eq!(elem.children.as_ref().unwrap().len(), 1);
            let child_node = &elem.children.as_ref().unwrap()[0];
            if let Node::Element(elem) = child_node {
                assert_eq!(elem.tag.as_ref().unwrap(), "pre");
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }
}
