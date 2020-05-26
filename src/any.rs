//! This module provides the [`AnyApp`](struct.AnyApp.html) wrapper which allows dynamically exchanging
//! components with different message types.
//!
//! ## Motivating Example
//!
//! Given a property pane which shows the user different properties based on his selection.
//! In a scene, any object could be selected and upon changing the selection, the property view
//! is exchanged.
//!
//! In this case, the [`AnyApp`](struct.AnyApp.html) may be used to dynamically switch between the
//! property views of different types, without wrapping each possible message type into an enum.
//!
//! ```
//! # use greenhorn::any::AnyApp;
//! # use greenhorn::{Render, App, Updated};
//! # use greenhorn::node::Node;
//! # use greenhorn::context::Context;
//! # use std::sync::{Mutex, Arc};
//! #
//! struct PropertyView {
//!     object: Object,
//!     view: Option<AnyApp>,
//! }
//!
//! impl PropertyView {
//!     fn show_object_properties(&mut self) {
//!         self.view = Some(AnyApp::new(self.object.clone()));
//!     }
//! }
//!
//! #[derive(Clone)]
//! struct Object(Arc<Mutex<ObjectData>>);
//!
//! struct ObjectData {
//!     // ...
//! }
//!
//! impl Render for Object {
//!     // ...
//! #    type Message = ();
//! #
//! #    fn render(&self) -> Node<Self::Message> {
//! #        unimplemented!()
//! #    }
//! }
//!
//! impl App for Object {
//!     // ...
//! #    fn update(&mut self, msg: Self::Message, ctx: Context<Self::Message>) -> Updated {
//! #        unimplemented!()
//! #    }
//! }
//! ```
//!
use crate::context::Context;
use crate::node::Node;
use crate::{App, Render, Updated};
use std::any::Any;

/// Wraps a type implementing [`App`](../trait.App.html), and as a consequence also [`Render`](../trait.Render.html)),
/// and exposes a new `App`/`Render` implementation with the dynamic [`AnyMsg`](type.AnyMsg.html) type.
///
/// ## Panics
///
/// In case update() is called with an invalid type, which is not convertible to the actual
/// message type of the underlying component.
pub struct AnyApp {
    inner: Box<dyn App<Message = AnyMsg> + Send>,
}

impl AnyApp {
    /// Construct an `AnyApp` object, consuming the underlying component.
    pub fn new<T, M>(app: T) -> AnyApp
    where
        T: 'static + App<Message = M> + Send,
        M: Any + Send + 'static,
    {
        AnyApp {
            inner: Box::new(AnyAppConverter { inner: app }),
        }
    }
}

/// Type alias for the dynamic message type used by `AnyApp`.
pub type AnyMsg = Box<dyn Any + Send>;

impl Render for AnyApp {
    type Message = AnyMsg;

    fn render(&self) -> Node<Self::Message> {
        self.inner.render()
    }
}

impl App for AnyApp {
    fn update(&mut self, msg: Self::Message, ctx: Context<Self::Message>) -> Updated {
        self.inner.update(msg, ctx)
    }
}

/// Internal type for erasing the type of an App implementation.
/// Note that when receiving a wrong message type, the `update()` function panics.
struct AnyAppConverter<T: App<Message = M>, M: Any + Send + 'static> {
    inner: T,
}

impl<T: App<Message = M>, M: Any + Send + 'static> Render for AnyAppConverter<T, M> {
    type Message = Box<dyn Any + Send + 'static>;

    fn render(&self) -> Node<Self::Message> {
        self.inner
            .render()
            .map(|x| Box::new(x) as Box<dyn Any + Send + 'static>)
    }
}

impl<T: App<Message = M>, M: Any + Send + 'static> App for AnyAppConverter<T, M> {
    fn update(&mut self, msg: Self::Message, ctx: Context<Self::Message>) -> Updated {
        let new_msg = *msg.downcast().unwrap();
        self.inner.update(
            new_msg,
            ctx.map(|x| Box::new(x) as Box<dyn Any + Send + 'static>),
        )
    }
}
