use std::cell::{Ref, RefCell, RefMut};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use crate::dom_event::DomEvent;
use crate::event::Subscription;
use crate::mailbox::Mailbox;
use crate::node_builder::NodeBuilder;
use crate::vdom::Attr;
use crate::Id;
use std::fmt::{Debug, Formatter, Error};

pub struct Updated {
    pub(crate) should_render: bool,
    pub(crate) components_render: Option<Vec<Id>>,
}

impl Updated {
    fn new() -> Updated {
        Updated {
            should_render: false,
            components_render: None,
        }
    }

    fn render(mut self) -> Self {
        self.should_render = true;
        self
    }

    fn invalidate(mut self, id: Id) -> Self {
        if let Some(ref mut ids) = self.components_render {
            ids.push(id)
        } else {
            self.components_render = Some(vec![id])
        }
        self
    }
}

impl From<bool> for Updated {
    fn from(x: bool) -> Self {
        Updated {
            should_render: x,
            components_render: None,
        }
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

struct ElementMapDirect<T: 'static, U> {
    fun: Arc<dyn Fn(T) -> U>,
    inner: NodeElement<T>,
}

impl<T, U> Debug for ElementMapDirect<T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.inner.fmt(f)
    }
}

impl<T: 'static, U: 'static> ElementMapDirect<T, U> {
    fn new_box(fun: Arc<dyn Fn(T) -> U>, inner: NodeElement<T>) -> Box<dyn ElementMap<U>> {
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
            .map(|x| x.map_arc(self.fun.clone()))
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
    fun: Arc<dyn Fn(T) -> U>,
    inner: Box<dyn ElementMap<T>>,
}

impl<T, U> Debug for ElementRemap<T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        std::fmt::Debug::fmt(&self.inner, f)
    }
}

impl<T: 'static, U: 'static> ElementRemap<T, U> {
    fn new_box(fun: Arc<dyn Fn(T) -> U>, inner: Box<dyn ElementMap<T>>) -> Box<dyn ElementMap<U>> {
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
            .map(|x| x.map_arc(self.fun.clone()))
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
    inner: Box<dyn ComponentMap<T>>,
}

impl<T> Debug for ComponentContainer<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        std::fmt::Debug::fmt(&self.inner, f)
    }
}

impl<T> ComponentMap<T> for ComponentContainer<T> {
    fn render(&self) -> Node<T> {
        self.inner.render()
    }

    fn id(&self) -> Id {
        self.inner.id()
    }
}

pub trait ComponentMap<T> : Debug {
    fn render(&self) -> Node<T>;
    fn id(&self) -> Id;
}

struct ComponentMapDirect<R: Render, U> {
    fun: Arc<dyn Fn(R::Message) -> U>,
    inner: Component<R>,
}

impl<R: Render, U> Debug for ComponentMapDirect<R, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.inner.fmt(f)
    }
}

impl<R: 'static + Render, U: 'static> ComponentMapDirect<R, U> {
    fn new_box(fun: Arc<dyn Fn(R::Message) -> U>, inner: Component<R>) -> Box<dyn ComponentMap<U>> {
        Box::new(Self { fun, inner })
    }
}

impl<R: 'static + Render, U: 'static> ComponentMap<U> for ComponentMapDirect<R, U> {
    fn render(&self) -> Node<U> {
        self.inner.render().map_arc(self.fun.clone())
    }

    fn id(&self) -> Id {
        self.inner.id()
    }
}

struct ComponentRemap<T, U> {
    fun: Arc<dyn Fn(T) -> U>,
    inner: Box<dyn ComponentMap<T>>,
}

impl<T, U> Debug for ComponentRemap<T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        std::fmt::Debug::fmt(&self.inner, f)
    }
}

impl<T: 'static, U: 'static> ComponentRemap<T, U> {
    fn new_box(
        fun: Arc<dyn Fn(T) -> U>,
        inner: Box<dyn ComponentMap<T>>,
    ) -> Box<dyn ComponentMap<U>> {
        Box::new(Self { fun, inner })
    }
}

impl<T: 'static, U: 'static> ComponentMap<U> for ComponentRemap<T, U> {
    fn render(&self) -> Node<U> {
        self.inner.render().map_arc(self.fun.clone())
    }

    fn id(&self) -> Id {
        self.inner.id()
    }
}

pub enum Node<T: 'static> {
    ElementMap(Box<dyn ElementMap<T>>),
    Component(Box<dyn ComponentMap<T>>),
    Text(String),
    Element(NodeElement<T>),
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

    pub fn map<U: 'static, F: 'static + Fn(T) -> U>(self, fun: F) -> Node<U> {
        let fun = Arc::new(fun);
        self.map_arc(fun)
    }

    pub(crate) fn map_arc<U: 'static>(self, fun: Arc<dyn Fn(T) -> U>) -> Node<U> {
        match self {
            Node::ElementMap(inner) => {
                let ret = ElementRemap::new_box(fun, inner);
                Node::ElementMap(ret)
            }
            Node::Component(inner) => Node::Component(ComponentRemap::new_box(fun, inner)),
            Node::Text(text) => Node::Text(text),
            Node::Element(elem) => Node::ElementMap(ElementMapDirect::new_box(fun, elem)),
            Node::EventSubscription(id, evt) => Node::EventSubscription(id, evt.map(fun)),
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
        }
    }

    pub fn id(&self) -> Id {
        match self {
            Node::ElementMap(inner) => inner.id(),
            Node::Component(inner) => inner.id(),
            Node::Text(_) => Id::empty(),
            Node::Element(elem) => elem.id,
            Node::EventSubscription(id, _) => *id,
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

pub struct Listener<T> {
    pub event_name: String,
    pub node_id: Id,
    pub fun: Arc<dyn Fn(DomEvent) -> T>,
    pub no_propagate: bool,
    pub prevent_default: bool,
}

// TODO: derive(Clone) failed for some reason?!
impl<T> Clone for Listener<T> {
    fn clone(&self) -> Self {
        Listener {
            event_name: self.event_name.clone(),
            node_id: self.node_id,
            fun: self.fun.clone(),
            no_propagate: self.no_propagate,
            prevent_default: self.prevent_default,
        }
    }
}

impl<T: 'static> Listener<T> {
    pub fn map<U: 'static>(self, fun: Arc<dyn Fn(T) -> U>) -> Listener<U> {
        let self_fun = self.fun;
        Listener {
            event_name: self.event_name,
            node_id: self.node_id,
            fun: Arc::new(move |e| fun((self_fun)(e))),
            no_propagate: self.no_propagate,
            prevent_default: self.prevent_default,
        }
    }

    pub fn call(&self, e: DomEvent) -> T {
        (self.fun)(e)
    }
}

pub struct Component<T: Render> {
    id: Id,
    comp: Rc<RefCell<T>>,
}

impl<T: Render> Debug for Component<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_fmt(format_args!("<Component {:?} />", self.id) )
    }
}

impl<T: Render> Clone for Component<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            comp: self.comp.clone(),
        }
    }
}

impl<T: 'static + Render> Component<T> {
    pub fn new(inner: T) -> Self {
        Self {
            id: Id::new(),
            comp: Rc::new(RefCell::new(inner)),
        }
    }

    pub fn borrow_mut(&mut self) -> RefMut<T> {
        self.comp.borrow_mut()
    }

    pub fn borrow(&self) -> Ref<T> {
        self.comp.borrow()
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn render(&self) -> Node<T::Message> {
        self.comp.deref().borrow().render()
    }

    pub fn map<R, F: Fn(&T) -> R>(&self, fun: F) -> R {
        let data = self.comp.deref().borrow();
        fun(&data)
    }

    pub fn update<R, F: FnOnce(&mut T) -> R>(&mut self, fun: F) -> R {
        let mut borrow = self.comp.deref().borrow_mut();
        let data = &mut borrow;
        fun(data)
    }
}

pub trait Render {
    type Message: 'static + Send;
    fn render(&self) -> Node<Self::Message>;

    fn html(&self) -> NodeBuilder<Self::Message> {
        NodeBuilder::new()
    }

    fn svg(&self) -> NodeBuilder<Self::Message> {
        NodeBuilder::new_with_ns("http://www.w3.org/2000/svg")
    }
}

pub trait App: Render {
    fn update(&mut self, msg: Self::Message, mailbox: Mailbox<Self::Message>) -> Updated;
    fn mount(&mut self, _mailbox: Mailbox<Self::Message>) {
    }
}
