use std::ops::DerefMut;

use crate::context::Context;
use crate::{Id, Render, App};
use std::fmt::{Debug, Formatter, Error};
use crate::node::{Node, NodeItems};
use std::collections::HashSet;
use std::collections::hash_map::RandomState;
use std::sync::{Arc, Mutex, MutexGuard};

pub struct Updated {
    pub(crate) should_render: bool,
    pub(crate) components_render: Option<Vec<Id>>,
}

impl Updated {
    pub fn new() -> Updated {
        Updated {
            should_render: false,
            components_render: None,
        }
    }

    pub fn yes() -> Updated {
        Updated {
            should_render: true,
            components_render: None
        }
    }

    pub fn no() -> Updated {
        Updated {
            should_render: false,
            components_render: None
        }
    }

    pub fn render(mut self) -> Self {
        self.should_render = true;
        self
    }

    pub fn invalidate<T: Render + Send>(mut self, component: &Component<T>) -> Self {
        if let Some(ref mut ids) = self.components_render {
            ids.push(component.id)
        } else {
            self.components_render = Some(vec![component.id])
        }
        self
    }

    pub fn merge(&mut self, other: Updated) {
        if other.should_render {
            self.should_render = true;
        } else if let Some(mut other_comps) = other.components_render {
            if let Some(comps) = self.components_render.as_mut() {
                comps.append(&mut other_comps);
            } else {
                self.components_render = Some(other_comps);
            }
        }
    }

    pub fn empty(&self) -> bool {
        !self.should_render && self.components_render.is_none()
    }
}

impl Default for Updated {
    fn default() -> Self {
        Self::new()
    }
}

impl Into<HashSet<Id>> for Updated {
    fn into(self) -> HashSet<Id, RandomState> {
        let mut ret = HashSet::new();
        if let Some(ids) = self.components_render {
            for id in ids {
                ret.insert(id);
            }
        }
        ret
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

impl From<Id> for Updated {
    fn from(id: Id) -> Self {
        Updated {
            should_render: false,
            components_render: Some(vec![id]),
        }
    }
}

pub struct Component<T: Render + Send> {
    id: Id,
    comp: Arc<Mutex<T>>,
}

impl<T: Render + Send> Debug for Component<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_fmt(format_args!("<Component {:?} />", self.id) )
    }
}

impl<T: Render + Send> Clone for Component<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            comp: self.comp.clone(),
        }
    }
}

impl<T: 'static + Render + Send> Component<T> {
    pub fn new(inner: T) -> Self {
        Self {
            id: Id::new(),
            comp: Arc::new(Mutex::new(inner)),
        }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        self.comp.lock().unwrap()
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn render(&self) -> Node<T::Message> {
        self.lock().render()
    }

    pub fn map<R, F: Fn(&T) -> R>(&self, fun: F) -> R {
        let data = self.lock();
        fun(&data)
    }

    pub fn update<R, F: FnOnce(&mut T) -> R>(&mut self, fun: F) -> R {
        let mut data = self.lock();
        fun(data.deref_mut())
    }

    pub fn mount(&self) -> Node<T::Message> {
        Node(NodeItems::Component(ComponentContainer::new(self)))
    }
}

impl<T: 'static + App + Send> Component<T> {
    pub fn update_app(&mut self, msg: T::Message, ctx: Context<T::Message>) -> Updated {
        let mut borrow = self.lock();
        let data = borrow.deref_mut();
        let mut ret = data.update(msg, ctx);
        if ret.should_render {
            // improve reporting accuracy
            ret.should_render = false;
            ret.components_render = Some(vec![self.id])
        }
        ret
    }
}

pub(crate) struct ComponentContainer<T: 'static + Send> {
    pub(crate) inner: Arc<Mutex<dyn ComponentMap<T>>>,
}

impl<T: 'static + Send> ComponentContainer<T> {
    fn new<U: 'static + Render<Message=T> + Send>(comp: &Component<U>) -> Self {
        let mounted = ComponentMapDirect {
            inner: comp.clone()
        };
        let inner = Arc::new(Mutex::new(mounted));
        ComponentContainer {
            inner
        }
    }
}

impl<T: 'static + Send> Clone for ComponentContainer<T> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

impl<T: 'static + Send> Debug for ComponentContainer<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        std::fmt::Debug::fmt(&self.inner, f)
    }
}

impl<T: 'static + Send> ComponentMap<T> for ComponentContainer<T> {
    fn render(&self) -> Node<T> {
        self.inner.lock().unwrap().render()
    }

    fn id(&self) -> Id {
        self.inner.lock().unwrap().id()
    }
}

pub(crate) trait ComponentMap<T: 'static + Send> : Debug + Send {
    fn render(&self) -> Node<T>;
    fn id(&self) -> Id;
}

pub(crate) struct ComponentMapDirect<R: Render + Send> {
    inner: Component<R>,
}

impl<R: Render + Send> Debug for ComponentMapDirect<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.inner.fmt(f)
    }
}

impl<R: 'static + Render + Send> ComponentMap<R::Message> for ComponentMapDirect<R> {
    fn render(&self) -> Node<R::Message> {
        self.inner.render()
    }

    fn id(&self) -> Id {
        self.inner.id()
    }
}

pub(crate) struct MappedComponent<T, U> {
    fun: Arc<Mutex<dyn Send + Fn(T) -> U>>,
    inner: Arc<Mutex<dyn ComponentMap<T>>>,
}

impl<T, U> Debug for MappedComponent<T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        std::fmt::Debug::fmt(&self.inner, f)
    }
}

impl<T: 'static + Send, U: 'static + Send> MappedComponent<T, U> {
    pub(crate) fn new_container(
        fun: Arc<Mutex<dyn Send + Fn(T) -> U>>,
        inner: Arc<Mutex<dyn ComponentMap<T>>>,
    ) -> ComponentContainer<U> {
        ComponentContainer {
            inner: Arc::new(Mutex::new(Self { fun, inner }))
        }
    }
}

impl<T: Send + 'static, U: Send + 'static> ComponentMap<U> for MappedComponent<T, U> {
    fn render(&self) -> Node<U> {
        self.inner.lock().unwrap().render().map_shared(self.fun.clone())
    }

    fn id(&self) -> Id {
        self.inner.lock().unwrap().id()
    }
}

