use crate::Id;
use std::fmt::{Debug, Formatter, Error};
use std::sync::{Arc, Mutex};
use crate::event::Subscription;
use crate::node_builder::NodeBuilder;
use crate::blob::Blob;
use crate::element::{Element, ElementMap, ElementRemap, ElementMapDirect, MappedElement};
use crate::component::{MappedComponent, ComponentContainer, ComponentMap};


pub enum Node<T: 'static> {
    ElementMap(MappedElement<T>),
    Component(ComponentContainer<T>),
    Text(String),
    Element(Element<T>),
    Blob(Blob),
    EventSubscription(Id, Subscription<T>),
}

impl<T: 'static> Debug for Node<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Node::ElementMap(x) => {std::fmt::Debug::fmt(&x, f)},
            Node::Component(x) => {std::fmt::Debug::fmt(&x, f)},
            Node::Text(text) => {f.write_str(&text)},
            Node::Element(elem) => {elem.fmt(f)},
            Node::EventSubscription(_, subs) => {subs.fmt(f)},
            Node::Blob(blob) => {blob.fmt(f)}
        }
    }
}

impl<T: 'static> Node<T> {
    pub fn html() -> NodeBuilder<T> {
        NodeBuilder::new()
    }

    pub fn svg() -> NodeBuilder<T> {
        NodeBuilder::new_with_ns("http://www.w3.org/2000/svg")
    }

    pub fn text<S: ToString>(data: S) -> Self {
        Node::Text(data.to_string())
    }

    pub fn map<U: 'static, F: 'static + Send + Fn(T) -> U>(self, fun: F) -> Node<U> {
        let fun: Arc<Mutex<Box<dyn 'static + Send + Fn(T) -> U>>> = Arc::new(Mutex::new(Box::new(fun)));
        self.map_shared(fun)
    }

    pub(crate) fn map_shared<U: 'static>(self, fun: Arc<Mutex<Box<dyn 'static + Send + Fn(T) -> U>>>) -> Node<U> {
        match self {
            Node::ElementMap(inner) => {
                let ret = ElementRemap::new_box(fun, inner.inner);
                Node::ElementMap(ret)
            }
            Node::Component(inner) => Node::Component(MappedComponent::new_container(fun, inner.inner)),
            Node::Text(text) => Node::Text(text),
            Node::Element(elem) => Node::ElementMap(ElementMapDirect::new_box(fun, elem)),
            Node::EventSubscription(id, evt) => Node::EventSubscription(id, evt.map(fun)),
            Node::Blob(blob) => Node::Blob(blob)
        }
    }

    /// Maps Node() objects without providing a mapping-functions.
    ///
    /// Panics in case there are listeners installed on this node or
    /// any child node.
    /// This allows mapping node-hierarchies without listeners efficiently without
    /// keeping the target message type around, for example when caching rendered nodes.
    pub fn empty_map<U: 'static>(self) -> Node<U> {
        match self {
            Node::ElementMap(_) => panic!(),
            Node::Component(_) => panic!(),
            Node::Text(x) => Node::Text(x),
            Node::Element(elem) => {
                if !elem.listeners.unwrap().is_empty() {
                    panic!();
                }
                let children = elem.children.map(
                    |mut x| x.drain(..).map(|x| x.empty_map()).collect()
                );
                Node::Element(Element {
                    id: elem.id,
                    tag: elem.tag,
                    attrs: elem.attrs,
                    js_events: elem.js_events,
                    listeners: Some(vec![]),
                    children,
                    namespace: elem.namespace
                })
            },
            Node::EventSubscription(_, _) => panic!(),
            Node::Blob(blob) => Node::Blob(blob),
        }
    }

    pub fn id(&self) -> Id {
        match self {
            Node::ElementMap(inner) => inner.id(),
            Node::Component(inner) => inner.id(),
            Node::Text(_) => Id::new_empty(),
            Node::Element(elem) => elem.id,
            Node::EventSubscription(id, _) => *id,
            Node::Blob(blob) => blob.id(),
        }
    }

    pub fn try_clone(&self) -> Option<Self> {
        match self {
            Node::Element(elem) => {
                if let Some(ret) = elem.try_clone() {
                    Some(Node::Element(ret))
                } else {
                    None
                }
            },
            Node::Text(txt) => Some(Node::Text(txt.clone())),
            Node::Blob(blob) => Some(Node::Blob(blob.clone())),
            _ => None
        }
    }
}

