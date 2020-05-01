use crate::listener::{Listener, Rpc};
use crate::node::Node;
use crate::vdom::Attr;
use crate::Id;
use std::fmt;
use std::fmt::{Debug, Error, Formatter};
use std::sync::{Arc, Mutex};

/// Element type to represent a DOM element
///
/// We use Option<...> to allow moving content out of an
/// Element instance.
pub(crate) struct Element<T: 'static + Send> {
    pub(crate) id: Id,
    pub(crate) tag: Option<String>, // The tag name. Must be present.
    pub(crate) attrs: Option<Vec<Attr>>, // Generic HTML attributes
    pub(crate) js_events: Option<Vec<Attr>>, // JS event handlers
    pub(crate) listeners: Option<Vec<Listener<T>>>, // Registered listeners
    pub(crate) children: Option<Vec<Node<T>>>, // child nodes
    pub(crate) namespace: Option<String>, // an optional namespace. If None the HTML namespace is assumed
    pub(crate) rpc: Option<Rpc<T>>,       // An RPC message handler for this node
}

impl<T: 'static + Send> Element<T> {
    /// Attempt to clone this element. Panics in case this element contains
    /// non-clonable children.
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
            id: self.id,
            tag: self.tag.clone(),
            attrs: self.attrs.clone(),
            js_events: self.js_events.clone(),
            listeners: self.listeners.clone(),
            children,
            namespace: self.namespace.clone(),
            rpc: self.rpc.clone(),
        })
    }
}

impl<T: 'static + Send> Debug for Element<T> {
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

/// Container to simplify type erasure of [Elements](struct.Element.html) with different
/// message type.
pub(crate) struct MappedElement<T: 'static + Send> {
    pub(crate) inner: Box<dyn ElementMap<T>>,
}

impl<T: 'static + Send> Debug for MappedElement<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ret = format!("MappedElement {{ inner: {:?} }}", self.inner);
        f.write_str(&ret)
    }
}

impl<T: 'static + Send> MappedElement<T> {
    fn new(elem: Box<dyn ElementMap<T>>) -> Self {
        Self { inner: elem }
    }
}

/// Trait interface for type erasure of [Elements](struct.Element.html)
///
/// All `.take_*()` functions are only callable once. Calling them more then once
/// will cause a panic.
pub(crate) trait ElementMap<T: 'static + Send>: Debug {
    fn take_listeners(&mut self) -> Vec<Listener<T>>;
    fn take_children(&mut self) -> Vec<Node<T>>;
    fn id(&self) -> Id;
    fn take_attrs(&mut self) -> Vec<Attr>;
    fn take_tag(&mut self) -> String;
    fn take_namespace(&mut self) -> Option<String>;
    fn take_js_events(&mut self) -> Vec<Attr>;
    fn take_rpc(&mut self) -> Option<Rpc<T>>;
}

impl<T: 'static + Send> ElementMap<T> for MappedElement<T> {
    fn take_listeners(&mut self) -> Vec<Listener<T>> {
        self.inner.take_listeners()
    }
    fn take_children(&mut self) -> Vec<Node<T>> {
        self.inner.take_children()
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
    fn take_js_events(&mut self) -> Vec<Attr> {
        self.inner.take_js_events()
    }
    fn take_rpc(&mut self) -> Option<Rpc<T>> {
        self.inner.take_rpc()
    }
}

impl<T: 'static + Send> ElementMap<T> for Element<T> {
    fn take_listeners(&mut self) -> Vec<Listener<T>> {
        self.listeners.take().unwrap()
    }
    fn take_children(&mut self) -> Vec<Node<T>> {
        self.children.take().unwrap()
    }
    fn id(&self) -> Id {
        self.id
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
    fn take_js_events(&mut self) -> Vec<Attr> {
        self.js_events.take().unwrap()
    }
    fn take_rpc(&mut self) -> Option<Rpc<T>> {
        self.rpc.take()
    }
}

pub(crate) struct ElementMapDirect<T: 'static + Send, U: 'static + Send> {
    fun: Arc<Mutex<dyn 'static + Send + Fn(T) -> U>>,
    inner: Element<T>,
}

impl<T: 'static + Send, U: 'static + Send> Debug for ElementMapDirect<T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.inner.fmt(f)
    }
}

impl<T: 'static + Send, U: 'static + Send> ElementMapDirect<T, U> {
    pub(crate) fn new_box(
        fun: Arc<Mutex<dyn 'static + Send + Fn(T) -> U>>,
        inner: Element<T>,
    ) -> MappedElement<U> {
        let ret = Box::new(ElementMapDirect { fun, inner });
        MappedElement { inner: ret }
    }
}

impl<T: 'static + Send, U: 'static + Send> ElementMap<U> for ElementMapDirect<T, U> {
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
        self.inner
            .js_events
            .take()
            .expect("js_events cannot be taken multiple times")
    }

    fn take_rpc(&mut self) -> Option<Rpc<U>> {
        self.inner.rpc.take().map(|x| x.map(self.fun.clone()))
    }
}

pub(crate) struct ElementRemap<T, U> {
    fun: Arc<Mutex<dyn 'static + Send + Fn(T) -> U>>,
    inner: Box<dyn ElementMap<T>>,
}

impl<T, U> Debug for ElementRemap<T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        std::fmt::Debug::fmt(&self.inner, f)
    }
}

impl<T: 'static + Send, U: 'static + Send> ElementRemap<T, U> {
    pub(crate) fn new_box(
        fun: Arc<Mutex<dyn 'static + Send + Fn(T) -> U>>,
        inner: Box<dyn ElementMap<T>>,
    ) -> MappedElement<U> {
        MappedElement::new(Box::new(ElementRemap { fun, inner }))
    }
}

impl<T: 'static + Send, U: 'static + Send> ElementMap<U> for ElementRemap<T, U> {
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
    fn take_js_events(&mut self) -> Vec<Attr> {
        self.inner.take_js_events()
    }

    fn take_rpc(&mut self) -> Option<Rpc<U>> {
        self.inner.take_rpc().map(|x| x.map(self.fun.clone()))
    }
}
