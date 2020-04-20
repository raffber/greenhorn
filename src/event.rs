use std::any::Any;
use std::marker::PhantomData;

use crate::Id;
use std::sync::{Arc, Mutex};
use std::fmt::{Debug, Formatter, Error};
use std::sync::atomic::AtomicPtr;

pub struct Emission {
    pub(crate) event_id: Id,
    pub(crate) data: Box<dyn Any>,
}

pub(crate) trait SubscriptionMap<T> : Send {
    fn call(&self, value: Box<dyn Any>) -> T;
    fn id(&self) -> Id;
}

struct MappedSubscription<U, T> {
    mapper: Arc<Mutex<dyn Send + Fn(U) -> T>>,
    child: Subscription<U>
}

impl<U: 'static, T: 'static> SubscriptionMap<T> for MappedSubscription<U, T> {
    fn call(&self, value: Box<dyn Any>) -> T {
        let mapper = self.mapper.lock().unwrap();
        let ret = self.child.call(value);
        (mapper)(ret)
    }

    fn id(&self) -> Id {
        self.child.id()
    }
}

struct SubscriptionHandler<T, V, F: Send + Fn(V) -> T> {
    handler: Mutex<F>,
    id: Id,
    a: std::marker::PhantomData<AtomicPtr<T>>,
    b: std::marker::PhantomData<AtomicPtr<V>>,
}

impl<T: 'static, V: 'static, F: Send + Fn(V) -> T> SubscriptionMap<T>
    for SubscriptionHandler<T, V, F>
{
    fn call(&self, value: Box<dyn Any>) -> T {
        let v = value.downcast::<V>().unwrap();
        (self.handler.lock().unwrap())(*v)
    }

    fn id(&self) -> Id {
        self.id.clone()
    }
}

pub struct Subscription<T>(Arc<Mutex<dyn SubscriptionMap<T>>>);


impl<T> Clone for Subscription<T> {
    fn clone(&self) -> Self {
        Subscription(self.0.clone())
    }
}


impl<T: 'static> Subscription<T> {
    pub(crate) fn map<U: 'static>(self, fun: Arc<Mutex<dyn Send + Fn(T) -> U>>) -> Subscription<U> {
        Subscription(Arc::new(Mutex::new(MappedSubscription {
            mapper: fun,
            child: self
        })))
    }

    pub(crate) fn call(&self, value: Box<dyn Any>) -> T {
        self.0.lock().unwrap().call(value)
    }

    pub(crate) fn id(&self) -> Id {
        self.0.lock().unwrap().id()
    }
}

impl<T: 'static> Debug for Subscription<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_fmt(format_args!( "<Subscription {:?} />", self.id()) )
    }
}


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

    pub fn emit<V: Any>(&self, value: V) -> Emission {
        let data = Box::new(value);
        Emission {
            event_id: self.id,
            data,
        }
    }

    pub fn subscribe<M: 'static, F: 'static + Send + Fn(T) -> M>(&self, fun: F) -> Subscription<M> {
        Subscription(Arc::new(Mutex::new(SubscriptionHandler {
            handler: Mutex::new(fun),
            id: self.id,
            a: PhantomData,
            b: PhantomData,
        })))
    }
}

impl<T: Any> Default for Event<T> {
    fn default() -> Self {
        Event::new()
    }
}

impl<T: Any> Clone for Event<T> {
    fn clone(&self) -> Self {
        Event {
            id: self.id,
            marker: PhantomData
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    #[derive(Debug)]
    enum Msg {
        Event(i32),
    }

    #[test]
    fn event_subscription() {
        let event = Event::<i32>::new();
        let subs = event.subscribe(Msg::Event);
        let emission = event.emit(3);
        let msg = subs.call(emission.data);
        assert_matches!(msg, Msg::Event(3));
    }
}