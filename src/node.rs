use crate::Id;
use std::fmt::{Debug, Formatter, Error};
use std::sync::{Arc, Mutex};
use crate::event::Subscription;
use crate::node_builder::NodeBuilder;
use crate::blob::Blob;
use crate::element::{Element, ElementMap, ElementRemap, ElementMapDirect, MappedElement};
use crate::component::{MappedComponent, ComponentContainer, ComponentMap};

pub struct Node<T: 'static>(pub(crate) NodeItems<T>);

pub(crate) enum NodeItems<T: 'static> {
    ElementMap(MappedElement<T>),
    Component(ComponentContainer<T>),
    Text(String),
    Element(Element<T>),
    Blob(Blob),
    EventSubscription(Id, Subscription<T>),
}

impl<T: 'static> Debug for Node<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match &self.0 {
            NodeItems::ElementMap(x) => {std::fmt::Debug::fmt(&x, f)},
            NodeItems::Component(x) => {std::fmt::Debug::fmt(&x, f)},
            NodeItems::Text(text) => {f.write_str(&text)},
            NodeItems::Element(elem) => {elem.fmt(f)},
            NodeItems::EventSubscription(_, subs) => {subs.fmt(f)},
            NodeItems::Blob(blob) => {blob.fmt(f)}
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
        Node(NodeItems::Text(data.to_string()))
    }

    pub fn map<U: 'static, F: 'static + Send + Fn(T) -> U>(self, fun: F) -> Node<U> {
        let fun: Arc<Mutex<Box<dyn 'static + Send + Fn(T) -> U>>> = Arc::new(Mutex::new(Box::new(fun)));
        self.map_shared(fun)
    }

    pub(crate) fn map_shared<U: 'static>(self, fun: Arc<Mutex<Box<dyn 'static + Send + Fn(T) -> U>>>) -> Node<U> {
        let ret = match self.0 {
            NodeItems::ElementMap(inner) => {
                let ret = ElementRemap::new_box(fun, inner.inner);
                NodeItems::ElementMap(ret)
            }
            NodeItems::Component(inner) => NodeItems::Component(MappedComponent::new_container(fun, inner.inner)),
            NodeItems::Text(text) => NodeItems::Text(text),
            NodeItems::Element(elem) => NodeItems::ElementMap(ElementMapDirect::new_box(fun, elem)),
            NodeItems::EventSubscription(id, evt) => NodeItems::EventSubscription(id, evt.map(fun)),
            NodeItems::Blob(blob) => NodeItems::Blob(blob)
        };
        Node(ret)
    }

    pub(crate) fn take_children(self) -> Vec<Node<T>> {
        match self.0 {
            NodeItems::ElementMap(mut x) => x.take_children(),
            NodeItems::Element(mut x) => x.take_children(),
            _ => panic!()
        }
    }

    /// Maps Node() objects without providing a mapping-functions.
    ///
    /// Panics in case there are listeners installed on this node or
    /// any child node.
    /// This allows mapping node-hierarchies without listeners efficiently without
    /// keeping the target message type around, for example when caching rendered nodes.
    pub fn empty_map<U: 'static>(self) -> Node<U> {
        match self.0 {
            NodeItems::ElementMap(_) => panic!(),
            NodeItems::Component(_) => panic!(),
            NodeItems::Text(x) => Node(NodeItems::Text(x)),
            NodeItems::Element(elem) => {
                if !elem.listeners.unwrap().is_empty() {
                    panic!();
                }
                let children = elem.children.map(
                    |mut x| x.drain(..).map(|x| x.empty_map()).collect()
                );
                Node(NodeItems::Element(Element {
                    id: elem.id,
                    tag: elem.tag,
                    attrs: elem.attrs,
                    js_events: elem.js_events,
                    listeners: Some(vec![]),
                    children,
                    namespace: elem.namespace
                }))
            },
            NodeItems::EventSubscription(_, _) => panic!(),
            NodeItems::Blob(blob) => Node(NodeItems::Blob(blob)),
        }
    }

    pub fn id(&self) -> Id {
        match &self.0 {
            NodeItems::ElementMap(inner) => inner.id(),
            NodeItems::Component(inner) => inner.id(),
            NodeItems::Text(_) => Id::new_empty(),
            NodeItems::Element(elem) => elem.id,
            NodeItems::EventSubscription(id, _) => *id,
            NodeItems::Blob(blob) => blob.id(),
        }
    }

    pub fn try_clone(&self) -> Option<Self> {
        match &self.0 {
            NodeItems::Element(elem) => {
                if let Some(ret) = elem.try_clone() {
                    Some(Node(NodeItems::Element(ret)))
                } else {
                    None
                }
            },
            NodeItems::Text(txt) => Some(Node(NodeItems::Text(txt.clone()))),
            NodeItems::Blob(blob) => Some(Node(NodeItems::Blob(blob.clone()))),
            _ => None
        }
    }
}

