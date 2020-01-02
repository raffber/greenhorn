use std::any::Any;
use std::marker::PhantomData;

use crate::Id;
use std::sync::Arc;
use std::fmt::{Debug, Formatter, Error};

pub(crate) struct Emission {
    pub(crate) event_id: Id,
    pub(crate) data: Box<dyn Any>,
}

pub trait SubscriptionMap<T> {
    fn call(&self, value: Box<dyn Any>) -> T;
    fn id(&self) -> Id;
}

struct SubscriptionMapImpl<U, T> {
    mapper: Arc<dyn Fn(U) -> T>,
    child: Subscription<U>,
}

impl<U: 'static, T: 'static> SubscriptionMap<T> for SubscriptionMapImpl<U, T> {
    fn call(&self, value: Box<dyn Any>) -> T {
        let ret = self.child.call(value);
        (self.mapper)(ret)
    }

    fn id(&self) -> Id {
        self.child.id()
    }
}

pub trait SubscriptionHandler<T> {
    fn call(&self, value: Box<dyn Any>) -> T;
}

struct SubscriptionHandlerImpl<T, V, F: Fn(V) -> T> {
    handler: F,
    a: std::marker::PhantomData<T>,
    b: std::marker::PhantomData<V>,
}

impl<T: 'static, V: 'static, F: Fn(V) -> T> SubscriptionHandler<T>
    for SubscriptionHandlerImpl<T, V, F>
{
    fn call(&self, value: Box<dyn Any>) -> T {
        let v = value.downcast::<V>().unwrap();
        (self.handler)(*v)
    }
}

pub enum Subscription<T> {
    Mapper(Arc<dyn SubscriptionMap<T>>),
    Handler(Id, Arc<dyn SubscriptionHandler<T>>),
}

impl<T> Clone for Subscription<T> {
    fn clone(&self) -> Self {
        match self {
            Subscription::Mapper(x) => {Subscription::Mapper(x.clone())},
            Subscription::Handler(id, x) => {Subscription::Handler(id.clone(), x.clone())},
        }
    }
}

impl<T: 'static> Debug for Subscription<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_fmt(format_args!( "<Subscription {:?} />", self.id()) )
    }
}

impl<T: 'static> Subscription<T> {
    pub(crate) fn map<U: 'static>(self, fun: Arc<dyn Fn(T) -> U>) -> Subscription<U> {
        Subscription::Mapper(Arc::new(SubscriptionMapImpl {
            mapper: fun,
            child: self,
        }))
    }

    pub(crate) fn call(&self, value: Box<dyn Any>) -> T {
        match self {
            Subscription::Mapper(map) => map.call(value),
            Subscription::Handler(_, fun) => fun.call(value),
        }
    }

    pub(crate) fn id(&self) -> Id {
        match self {
            Subscription::Mapper(map) => map.id(),
            Subscription::Handler(id, _) => *id,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Event<T: Any> {
    id: Id,
    marker: PhantomData<T>,
}

impl<T: Any> Event<T> {
    pub fn new() -> Event<T> {
        Event {
            id: Id::new(),
            marker: PhantomData,
        }
    }

    pub(crate) fn emit<V: Any>(&self, value: V) -> Emission {
        let data = Box::new(value);
        Emission {
            event_id: self.id,
            data,
        }
    }

    pub fn subscribe<M: 'static, F: 'static + Fn(T) -> M>(&self, fun: F) -> Subscription<M> {
        Subscription::Handler(
            self.id,
            Arc::new(SubscriptionHandlerImpl {
                handler: fun,
                a: PhantomData,
                b: PhantomData,
            }),
        )
    }
}

impl<T: Any> Default for Event<T> {
    fn default() -> Self {
        Event::new()
    }
}

