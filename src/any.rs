//! This module provides the [AnyApp](struct.AnyApp.html) wrapper which allows dynamically exchanging
//! components with different message types.
//!
//! ## Motivating Example
//!
//! Given a property pane which shows the user different properties based on his selection.
//! In a scene, any objects could be selected and upon changing the selection, the property view
//! is exchanged.
//!
//! In this case, the [AnyApp](struct.AnyApp.html) can be used to dynamically switch between the
//! property views, without wrapping each possible message type into an enum.
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
//!
//!

use crate::{App, Render, Updated};
use std::any::Any;
use crate::node::Node;
use crate::context::Context;

pub type AnyMsg = Box<dyn Any + Send>;

pub struct AnyApp {
    inner: Box<dyn App<Message=AnyMsg> + Send>,
}

impl AnyApp {
    pub fn new<T, M>(app: T) -> AnyApp
        where
            T: 'static + App<Message=M> + Send,
            M: Any + Send + 'static {
        AnyApp {
            inner: Box::new(AnyAppConverter { inner: app })
        }
    }
}

impl Render for AnyApp {
    type Message = Box<dyn Any + Send>;

    fn render(&self) -> Node<Self::Message> {
        self.inner.render()
    }
}

impl App for AnyApp {
    fn update(&mut self, msg: Self::Message, ctx: Context<Self::Message>) -> Updated {
        self.inner.update(msg, ctx)
    }
}

struct AnyAppConverter<T: App<Message=M>, M: Any + Send + 'static> {
    inner: T,
}

impl<T: App<Message=M>, M: Any + Send + 'static> Render for AnyAppConverter<T, M> {
    type Message = Box<dyn Any + Send + 'static>;

    fn render(&self) -> Node<Self::Message> {
        self.inner.render().map(|x| Box::new(x) as Box<dyn Any + Send + 'static>)
    }
}

impl<T: App<Message=M>, M: Any + Send + 'static> App for AnyAppConverter<T, M> {
    fn update(&mut self, msg: Self::Message, ctx: Context<Self::Message>) -> Updated {
        let new_msg = *msg.downcast().unwrap();
        self.inner.update(new_msg, ctx.map(|x| Box::new(x) as Box<dyn Any + Send + 'static>))
    }
}
