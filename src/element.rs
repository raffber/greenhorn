use crate::Id;
use crate::vdom::Attr;
use crate::listener::Listener;
use crate::node::Node;
use std::fmt::{Debug, Formatter, Error};
use std::sync::{Arc, Mutex};

pub struct Element<T: 'static> {
    pub id: Id,
    pub tag: Option<String>,
    pub attrs: Option<Vec<Attr>>,
    pub js_events: Option<Vec<Attr>>,
    pub listeners: Option<Vec<Listener<T>>>,
    pub children: Option<Vec<Node<T>>>,
    pub namespace: Option<String>,
}

impl<T: 'static> Element<T> {
    pub(crate) fn try_clone(&self) -> Option<Self> {
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
            js_events: self.js_events.clone(),
            listeners: self.listeners.clone(),
            children,
            namespace: self.namespace.clone()
        })
    }
}

impl<T> Debug for Element<T> {
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
    fn take_js_events(&mut self) -> Vec<Attr>;
}

impl<T: 'static> ElementMap<T> for Element<T> {
    fn take_listeners(&mut self) -> Vec<Listener<T>> { self.listeners.take().unwrap() }
    fn take_children(&mut self) -> Vec<Node<T>> { self.children.take().unwrap() }
    fn id(&self) -> Id { self.id.clone() }
    fn take_attrs(&mut self) -> Vec<Attr> { self.attrs.take().unwrap() }
    fn take_tag(&mut self) -> String { self.tag.take().unwrap() }
    fn take_namespace(&mut self) -> Option<String> { self.namespace.take() }
    fn take_js_events(&mut self) -> Vec<Attr> { self.js_events.take().unwrap() }
}

pub(crate) struct ElementMapDirect<T: 'static, U> {
    fun: Arc<Mutex<Box<dyn 'static + Send + Fn(T) -> U>>>,
    inner: Element<T>,
}

impl<T, U> Debug for ElementMapDirect<T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.inner.fmt(f)
    }
}

impl<T: 'static, U: 'static> ElementMapDirect<T, U> {
    pub(crate) fn new_box(fun: Arc<Mutex<Box<dyn 'static + Send + Fn(T) -> U>>>, inner: Element<T>) -> Box<dyn ElementMap<U>> {
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

    fn take_js_events(&mut self) -> Vec<Attr> {
        self.inner.js_events.take().expect("js_events cannot be taken multiple times")
    }
}

pub(crate) struct ElementRemap<T, U> {
    fun: Arc<Mutex<Box<dyn 'static + Send + Fn(T) -> U>>>,
    inner: Box<dyn ElementMap<T>>,
}

impl<T, U> Debug for ElementRemap<T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        std::fmt::Debug::fmt(&self.inner, f)
    }
}

impl<T: 'static, U: 'static> ElementRemap<T, U> {
    pub(crate) fn new_box(fun: Arc<Mutex<Box<dyn 'static + Send + Fn(T) -> U>>>, inner: Box<dyn ElementMap<T>>) -> Box<dyn ElementMap<U>> {
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

    fn id(&self) -> Id { self.inner.id() }
    fn take_attrs(&mut self) -> Vec<Attr> { self.inner.take_attrs() }
    fn take_tag(&mut self) -> String { self.inner.take_tag() }
    fn take_namespace(&mut self) -> Option<String> { self.inner.take_namespace() }
    fn take_js_events(&mut self) -> Vec<Attr> { self.inner.take_js_events() }
}
