use std::cell::{Ref, RefCell, RefMut};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use crate::dom_event::DomEvent;
use crate::event::Subscription;
use crate::mailbox::Mailbox;
use crate::node_builder::NodeBuilder;
use crate::Id;
use crate::vdom::Attr;

pub struct Updated {
    should_render: bool,
    components_render: Option<Vec<Id>>,
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

pub struct NodeElement<T> {
    pub id: Id,
    pub tag: Option<String>,
    pub attrs: Option<Vec<Attr>>,
    pub listeners: Option<Vec<Listener<T>>>,
    pub children: Option<Vec<Node<T>>>,
    pub namespace: Option<String>,
}

pub trait ElementMap<T> {
    fn take_listeners(&mut self) -> Vec<Listener<T>>;
    fn take_children(&mut self) -> Vec<Node<T>>;
    fn id(&self) -> Id;
    fn take_attrs(&mut self) -> Vec<Attr>;
    fn take_tag(&mut self) -> String;
    fn take_namespace(&mut self) -> Option<String>;
}

struct ElementMapDirect<T, U> {
    fun: Arc<dyn Fn(T) -> U>,
    inner: NodeElement<T>,
}

impl<T: 'static, U: 'static> ElementMapDirect<T, U> {
    fn new(fun: Arc<dyn Fn(T) -> U>, inner: NodeElement<T>) -> Box<dyn ElementMap<U>> {
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
        self.inner.id.clone()
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

impl<T: 'static, U: 'static> ElementRemap<T, U> {
    fn new(fun: Arc<dyn Fn(T) -> U>, inner: Box<dyn ElementMap<T>>) -> Box<dyn ElementMap<U>> {
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
    inner: Box<dyn ComponentMap<T>>
}

impl<T> ComponentMap<T> for ComponentContainer<T> {
    fn render(&self) -> Node<T> {
        self.inner.render()
    }

    fn id(&self) -> Id {
        self.inner.id()
    }
}

pub trait ComponentMap<T> {
    fn render(&self) -> Node<T>;
    fn id(&self) -> Id;
}

struct ComponentMapDirect<R: Render, U> {
    fun: Arc<dyn Fn(R::Message) -> U>,
    inner: Component<R>,
}

impl<R: 'static + Render, U: 'static> ComponentMapDirect<R, U> {
    fn new(fun: Arc<dyn Fn(R::Message) -> U>, inner: Component<R>) -> Box<dyn ComponentMap<U>> {
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

impl<T: 'static, U: 'static> ComponentRemap<T, U> {
    fn new(fun: Arc<dyn Fn(T) -> U>, inner: Box<dyn ComponentMap<T>>) -> Box<dyn ComponentMap<U>> {
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

pub enum Node<T> {
    ElementMap(Box<dyn ElementMap<T>>),
    Component(Box<dyn ComponentMap<T>>),
    Text(String),
    Element(NodeElement<T>),
    EventSubscription(Id, Subscription<T>),
}

impl<T: 'static> Node<T> {
    pub fn map<U: 'static, F: 'static + Fn(T) -> U>(self, fun: F) -> Node<U> {
        let fun = Arc::new(fun);
        self.map_arc(fun)
    }

    pub fn map_arc<U: 'static>(self, fun: Arc<dyn Fn(T) -> U>) -> Node<U> {
        match self {
            Node::ElementMap(inner) => {
                let ret = ElementRemap::new(fun, inner);
                Node::ElementMap(ret)
            }
            Node::Component(inner) => Node::Component(ComponentRemap::new(fun, inner)),
            Node::Text(text) => Node::Text(text),
            Node::Element(elem) => Node::ElementMap(ElementMapDirect::new(fun, elem)),
            Node::EventSubscription(id, evt) => Node::EventSubscription(id.clone(), evt.map(fun)),
        }
    }

    pub fn id(&self) -> Id {
        match self {
            Node::ElementMap(inner) => inner.id(),
            Node::Component(inner) => inner.id(),
            Node::Text(_) => Id::empty(),
            Node::Element(elem) => elem.id,
            Node::EventSubscription(id, _) => id.clone(),
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
        Node::EventSubscription(Id::new(), value)
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

impl<T: Render> Clone for Component<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
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
        self.id.clone()
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
        let ret = fun(data);
        ret
    }
}

pub trait Render {
    type Message: 'static;
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
}
