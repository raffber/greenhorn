use std::ops::DerefMut;

use crate::context::Context;
use crate::Id;
use std::fmt::{Debug, Formatter, Error};
use crate::node::Node;
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

    pub fn invalidate(mut self, id: Id) -> Self {
        if let Some(ref mut ids) = self.components_render {
            ids.push(id)
        } else {
            self.components_render = Some(vec![id])
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

pub struct Component<T: Render> {
    id: Id,
    comp: Arc<Mutex<T>>,
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
}

impl<T: 'static + App> Component<T> {
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

pub trait Render {
    type Message: 'static + Send;
    fn render(&self) -> Node<Self::Message>;
}

pub trait App: Render {
    fn update(&mut self, msg: Self::Message, ctx: Context<Self::Message>) -> Updated;
    fn mount(&mut self, _ctx: Context<Self::Message>) {
    }
}


pub struct ComponentContainer<T> {
    pub(crate) inner: Arc<Mutex<Box<dyn ComponentMap<T>>>>,
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

pub(crate) struct ComponentMapDirect<R: Send + Render, U> {
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

pub(crate) struct ComponentRemap<T, U> {
    fun: Arc<Mutex<Box<dyn Send + Fn(T) -> U>>>,
    inner: Arc<Mutex<Box<dyn ComponentMap<T>>>>,
}

impl<T, U> Debug for ComponentRemap<T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        std::fmt::Debug::fmt(&self.inner, f)
    }
}

impl<T: 'static, U: 'static> ComponentRemap<T, U> {
    pub(crate) fn new_container(
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

