//! This module defines the [Event](struct.Event.html) type which facilitates communication
//! between components.
//!
//! During the `render()` phase, it is possible to subscribe to [Event](struct.Event.html) objects.
//! Event subscriptions are part of the frontend state and are represented in [Node](../node/struct.Node.html)
//! objects.
//! In case an [Event](struct.Event.html) is `.emit()`-ed during the `update()` phase
//! the respective event subscriptions are queried and trigger a new `update()` cycle.
//!
//! Oftentimes, components may want to inform other components about a change of their state.
//! In such a case it may expose [Event](struct.Event.html) objects, to which other components
//! can subscribe to.
//!
//! Note that events can only be subsribed to once, since the data of the event emission is
//! moved into the `update()` cycle of the application.
//!
//! ## Example
//!
//! ```
//! # use greenhorn::{Render, App, Updated};
//! # use greenhorn::node::Node;
//! # use greenhorn::dom::DomEvent;
//! # use greenhorn::event::Event;
//! # use greenhorn::context::Context;
//! # use greenhorn::html;
//! #
//! struct CheckBox {
//!     change: Event<bool>,
//!     checked: bool,
//! }
//!
//! enum CheckBoxMsg {
//!     Click(DomEvent), KeyDown(DomEvent)
//! }
//!
//! impl App for CheckBox {
//!     fn update(&mut self, msg: Self::Message, ctx: Context<Self::Message>) -> Updated {
//!         match msg {
//!             CheckBoxMsg::Click(evt) => {
//!                 self.checked = !self.checked;
//!                 ctx.emit(&self.change, self.checked);
//!                 Updated::yes()
//!             }
//!             CheckBoxMsg::KeyDown(evt) => {
//!                 let key = evt.into_keyboard().unwrap().key;
//!                 if key == "Enter" || key == "Space" {
//!                     self.checked = !self.checked;
//!                     ctx.emit(&self.change, self.checked);
//!                     Updated::yes()
//!                 } else {
//!                     Updated::no()
//!                 }
//!             }
//!         }
//!     }
//! }
//!
//! impl Render for CheckBox {
//!     type Message = CheckBoxMsg;
//!
//!     fn render(&self) -> Node<Self::Message> {
//!         // render a fancy checkbox and subscribe to DOM events...
//! #        Node::html().elem("div").build()
//!     }
//!
//! }
//!
//! struct Parent {
//!     checkbox: CheckBox
//! }
//!
//! enum ParentMsg {
//!     // ...
//!     CheckBox(CheckBoxMsg),
//!     CheckedChanged(bool),
//! }
//!
//! impl Render for Parent {
//!     type Message = ParentMsg;
//!
//!     fn render(&self) -> Node<Self::Message> {
//!         html!(<div>
//!                 {self.checkbox.render().map(ParentMsg::CheckBox)}
//!                 {self.checkbox.change.subscribe(ParentMsg::CheckedChanged)}
//!             </div>
//!         ).into()
//!     }
//! }
//! ```

use std::any::Any;
use std::marker::PhantomData;

use crate::Id;
use std::fmt::{Debug, Error, Formatter};
use std::sync::atomic::AtomicPtr;
use std::sync::{Arc, Mutex};

/// This type is created if a [Event](struct.Event.html) is `emit()`-ed.
///
/// It wraps the emitted event data. The application runtime is responsible for
/// associating it with an applicable event subscription.
pub struct Emission {
    pub(crate) event_id: Id,
    pub(crate) data: Box<dyn Any>,
}

pub(crate) trait SubscriptionMap<T>: Send {
    fn call(&self, value: Box<dyn Any>) -> T;
    fn id(&self) -> Id;
}

struct MappedSubscription<U, T> {
    mapper: Arc<Mutex<dyn Send + Fn(U) -> T>>,
    child: Subscription<U>,
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
        self.id
    }
}

/// Represents the result of an `Event.subscribe()` call.
///
/// `Subscription` objects are converted into [Node](../node/struct.Node.html) objects
/// which are generated by the `render()` phase of the application.
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
            child: self,
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
        f.write_fmt(format_args!("<Subscription {:?} />", self.id()))
    }
}

/// Events allow connecting different components with each other.
///
/// Components may `emit()` events during the `update()` cycle of an application and they
/// subscribe to events during the `render()` phase. Emitting an event which has been subscribed
/// to triggers a new `update()` cycle.
///
/// For more information refer to the [module-level documentation](index.html).
#[derive(Debug)]
pub struct Event<T: Any> {
    id: Id,
    marker: PhantomData<T>,
}

impl<T: Any> Event<T> {
    /// Create a new event.
    pub fn new() -> Event<T> {
        Event {
            id: Id::new(),
            marker: PhantomData,
        }
    }

    /// Emit an event. This is typically done during the `update()` phase.
    ///
    /// Also refer to [Context::emit()](../context/struct.Context.html).
    pub fn emit<V: Any>(&self, value: V) -> Emission {
        let data = Box::new(value);
        Emission {
            event_id: self.id,
            data,
        }
    }

    /// Subscribe to an event. This is typically done during the `update()` phase.
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
            marker: PhantomData,
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
