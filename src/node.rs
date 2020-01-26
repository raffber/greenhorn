use crate::{Id, Render, Component};
use crate::vdom::Attr;
use std::fmt::{Debug, Formatter, Error};
use crate::listener::Listener;
use std::sync::{Arc, Mutex};
use crate::event::Subscription;
use crate::node_builder::{NodeBuilder, BlobBuilder};

pub enum Node<T: 'static> {
    ElementMap(Box<dyn ElementMap<T>>),
    Component(ComponentContainer<T>),
    Text(String),
    Element(NodeElement<T>),
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

    pub fn map<U: 'static, F: 'static + Send + Fn(T) -> U>(self, fun: F) -> Node<U> {
        let fun: Arc<Mutex<Box<dyn 'static + Send + Fn(T) -> U>>> = Arc::new(Mutex::new(Box::new(fun)));
        self.map_shared(fun)
    }

    pub(crate) fn map_shared<U: 'static>(self, fun: Arc<Mutex<Box<dyn 'static + Send + Fn(T) -> U>>>) -> Node<U> {
        match self {
            Node::ElementMap(inner) => {
                let ret = ElementRemap::new_box(fun, inner);
                Node::ElementMap(ret)
            }
            Node::Component(inner) => Node::Component(ComponentRemap::new_container(fun, inner.inner)),
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
                if elem.listeners.unwrap().len() != 0 {
                    panic!();
                }
                let children = elem.children.map(
                    |mut x| x.drain(..).map(|x| x.empty_map()).collect()
                );
                Node::Element(NodeElement {
                    id: elem.id,
                    tag: elem.tag,
                    attrs: elem.attrs,
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
            Node::Text(_) => Id::empty(),
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

pub struct NodeElement<T: 'static> {
    pub id: Id,
    pub tag: Option<String>,
    pub attrs: Option<Vec<Attr>>,
    pub listeners: Option<Vec<Listener<T>>>,
    pub children: Option<Vec<Node<T>>>,
    pub namespace: Option<String>,
}

impl<T: 'static> NodeElement<T> {
    fn try_clone(&self) -> Option<Self> {
        let children = if let Some(children) = self.children.as_ref() {
            let mut new_children = Vec::with_capacity(children.len());
            for child in children {
                if let Some(cloned) = child.try_clone() {
                    new_children.push(cloned)
                } else {
                    return None;
                }
            }
            Some(new_children)
        } else {
            None
        };
        Some(Self {
            id: self.id.clone(),
            tag: self.tag.clone(),
            attrs: self.attrs.clone(),
            listeners: self.listeners.clone(),
            children,
            namespace: self.namespace.clone()
        })
    }
}

impl<T> Debug for NodeElement<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        if let Some(tag) = &self.tag {
            let _ = f.write_fmt(format_args!("<{} ", tag));
        }
        if let Some(attrs) = &self.attrs {
            for attr in attrs {
                let _ = f.write_fmt(format_args!("{} = \"{}\" ", &attr.key, &attr.value));
            }
        }
        let _ = f.write_str(">");
        if let Some(children) = self.children.as_ref() {
            for child in children {
                let _ = f.write_str("\n");
                let _ = child.fmt(f);
            }
        }
        f.write_str("</>")
    }
}

pub trait ElementMap<T> : Debug {
    fn take_listeners(&mut self) -> Vec<Listener<T>>;
    fn take_children(&mut self) -> Vec<Node<T>>;
    fn id(&self) -> Id;
    fn take_attrs(&mut self) -> Vec<Attr>;
    fn take_tag(&mut self) -> String;
    fn take_namespace(&mut self) -> Option<String>;
}

impl<T: 'static> ElementMap<T> for NodeElement<T> {
    fn take_listeners(&mut self) -> Vec<Listener<T>> {
        self.listeners.take().unwrap()
    }

    fn take_children(&mut self) -> Vec<Node<T>> {
        self.children.take().unwrap()
    }

    fn id(&self) -> Id {
        self.id.clone()
    }

    fn take_attrs(&mut self) -> Vec<Attr> {
        self.attrs.take().unwrap()
    }

    fn take_tag(&mut self) -> String {
        self.tag.take().unwrap()
    }

    fn take_namespace(&mut self) -> Option<String> {
        self.namespace.take()
    }
}

struct ElementMapDirect<T: 'static, U> {
    fun: Arc<Mutex<Box<dyn 'static + Send + Fn(T) -> U>>>,
    inner: NodeElement<T>,
}

impl<T, U> Debug for ElementMapDirect<T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.inner.fmt(f)
    }
}

impl<T: 'static, U: 'static> ElementMapDirect<T, U> {
    fn new_box(fun: Arc<Mutex<Box<dyn 'static + Send + Fn(T) -> U>>>, inner: NodeElement<T>) -> Box<dyn ElementMap<U>> {
        Box::new(ElementMapDirect { fun, inner })
    }
}

impl<T: 'static, U: 'static> ElementMap<U> for ElementMapDirect<T, U> {
    fn take_listeners(&mut self) -> Vec<Listener<U>> {
        self.inner
            .listeners
            .take()
            .expect("listeners taken multiple times")
            .drain(..)
            .map(|x| x.map(self.fun.clone()))
            .collect()
    }

    fn take_children(&mut self) -> Vec<Node<U>> {
        self.inner
            .children
            .take()
            .expect("children taken multiple times")
            .drain(..)
            .map(|x| x.map_shared(self.fun.clone()))
            .collect()
    }

    fn id(&self) -> Id {
        self.inner.id
    }

    fn take_attrs(&mut self) -> Vec<Attr> {
        self.inner.attrs.take().expect("attrs taken multiple times")
    }

    fn take_tag(&mut self) -> String {
        self.inner.tag.take().expect("name taken multiple times")
    }

    fn take_namespace(&mut self) -> Option<String> {
        self.inner.namespace.take()
    }
}

struct ElementRemap<T, U> {
    fun: Arc<Mutex<Box<dyn 'static + Send + Fn(T) -> U>>>,
    inner: Box<dyn ElementMap<T>>,
}

impl<T, U> Debug for ElementRemap<T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        std::fmt::Debug::fmt(&self.inner, f)
    }
}

impl<T: 'static, U: 'static> ElementRemap<T, U> {
    fn new_box(fun: Arc<Mutex<Box<dyn 'static + Send + Fn(T) -> U>>>, inner: Box<dyn ElementMap<T>>) -> Box<dyn ElementMap<U>> {
        Box::new(ElementRemap { fun, inner })
    }
}

impl<T: 'static, U: 'static> ElementMap<U> for ElementRemap<T, U> {
    fn take_listeners(&mut self) -> Vec<Listener<U>> {
        self.inner
            .take_listeners()
            .drain(..)
            .map(|x| x.map(self.fun.clone()))
            .collect()
    }

    fn take_children(&mut self) -> Vec<Node<U>> {
        self.inner
            .take_children()
            .drain(..)
            .map(|x| x.map_shared(self.fun.clone()))
            .collect()
    }

    fn id(&self) -> Id {
        self.inner.id()
    }

    fn take_attrs(&mut self) -> Vec<Attr> {
        self.inner.take_attrs()
    }

    fn take_tag(&mut self) -> String {
        self.inner.take_tag()
    }

    fn take_namespace(&mut self) -> Option<String> {
        self.inner.take_namespace()
    }
}

pub struct ComponentContainer<T> {
    inner: Arc<Mutex<Box<dyn ComponentMap<T>>>>,
}

impl<T> Clone for ComponentContainer<T> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

impl<T> ComponentContainer<T> {
   pub(crate) fn new(inner: Arc<Mutex<Box<dyn ComponentMap<T>>>>) -> Self {
        ComponentContainer {
            inner
        }
   }
}

impl<T> Debug for ComponentContainer<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        std::fmt::Debug::fmt(&self.inner, f)
    }
}

impl<T> ComponentMap<T> for ComponentContainer<T> {
    fn render(&self) -> Node<T> {
        self.inner.lock().unwrap().render()
    }

    fn id(&self) -> Id {
        self.inner.lock().unwrap().id()
    }
}

pub trait ComponentMap<T> : Debug + Send {
    fn render(&self) -> Node<T>;
    fn id(&self) -> Id;
}

struct ComponentMapDirect<R: Send + Render, U> {
    fun: Arc<Mutex<Box<dyn Send + Fn(R::Message) -> U>>>,
    inner: Component<R>,
}

impl<R: Send + Render, U> Debug for ComponentMapDirect<R, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.inner.fmt(f)
    }
}

impl<R: 'static + Send + Render, U: 'static> ComponentMapDirect<R, U> {
    fn new_box(fun: Arc<Mutex<Box<dyn 'static + Send + Fn(R::Message) -> U>>>, inner: Component<R>) -> Box<dyn ComponentMap<U>> {
        Box::new(Self { fun, inner })
    }
}

impl<R: 'static + Send + Render, U: 'static> ComponentMap<U> for ComponentMapDirect<R, U> {
    fn render(&self) -> Node<U> {
        self.inner.lock().render().map_shared(self.fun.clone())
    }

    fn id(&self) -> Id {
        self.inner.id()
    }
}

struct ComponentRemap<T, U> {
    fun: Arc<Mutex<Box<dyn Send + Fn(T) -> U>>>,
    inner: Arc<Mutex<Box<dyn ComponentMap<T>>>>,
}

impl<T, U> Debug for ComponentRemap<T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        std::fmt::Debug::fmt(&self.inner, f)
    }
}

impl<T: 'static, U: 'static> ComponentRemap<T, U> {
    fn new_container(
        fun: Arc<Mutex<Box<dyn Send + Fn(T) -> U>>>,
        inner: Arc<Mutex<Box<dyn ComponentMap<T>>>>,
    ) -> ComponentContainer<U> {
        ComponentContainer {
            inner: Arc::new(Mutex::new(Box::new(Self { fun, inner })))
        }
    }
}

impl<T: 'static, U: 'static> ComponentMap<U> for ComponentRemap<T, U> {
    fn render(&self) -> Node<U> {
        self.inner.lock().unwrap().render().map_shared(self.fun.clone())
    }

    fn id(&self) -> Id {
        self.inner.lock().unwrap().id()
    }
}

pub struct BlobData {
    hash: u64,
    id: Id,
    data: Vec<u8>,
    mime_type: String,
}

#[derive(Clone)]
pub struct Blob {
    inner: Arc<BlobData>
}

impl Blob {
    pub fn build(id: Id, hash: u64) -> BlobBuilder {
        BlobBuilder {
            id,
            hash,
            mime_type: "".to_string(),
            data: vec![]
        }
    }

    pub fn id(&self) -> Id {
        self.inner.id
    }

    pub fn hash(&self) -> u64 {
        self.inner.hash
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.inner.data
    }

    pub fn mime_type(&self) -> &str {
        &self.inner.mime_type
    }
}

impl Debug for Blob {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str(&format!("<Blob id={} hash={}>", self.id(), self.hash()))
    }
}

impl From<BlobBuilder> for Blob {
    fn from(builder: BlobBuilder) -> Self {
        Blob {
            inner: Arc::new(BlobData {
                hash: builder.hash,
                id: builder.id,
                data: builder.data,
                mime_type: builder.mime_type
            }),
        }
    }
}

impl<T: 'static> From<Blob> for Node<T> {
    fn from(blob: Blob) -> Self {
        Node::Blob(blob)
    }
}
