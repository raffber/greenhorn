#![allow(dead_code)]
#![recursion_limit = "256"]

///
/// Greenhorn - API Documentation
///
/// Greenhorn is a rust library for building desktop applications with web technologies in (almost)
/// pure rust.
///
/// This is accomplished by separating the application into a server-side process and web view.
/// While most HTML-based desktop applications leave the synchronization of state up to the application logic, this
/// library syncs its state at DOM-level.
/// Thus, the user may implement the application logic purely in server-side rust.
/// This facilitates the integration of the frontend with system services and simplifies application development
/// considerably.
///
///

use serde::{Deserialize, Serialize};
use std::cmp::Eq;
use std::convert::From;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};

pub mod component;
pub mod dom;
pub mod event;
pub mod context;
pub mod node_builder;
pub mod pipe;
pub mod runtime;
pub mod service;
mod vdom;
pub mod websockets;
pub mod node;
pub mod listener;
pub mod blob;
pub mod element;
pub mod components;
pub mod dialog;
pub mod services;
pub mod any;

/// Prelude, `use greehorn::prelude::*` imports the most important symbols for quick access
///
/// This module allows importing the most common types for building a greenhorn powered application
/// ```
/// # #![allow(unused_imports)]
/// use greenhorn::prelude::*;
/// ```
pub mod prelude {
    pub use crate::component::{Component, Updated};
    pub use crate::{App, Render};
    pub use crate::node::Node;
    pub use crate::dom::{KeyboardEvent, WheelEvent, MouseEvent, DomEvent, ChangeEvent, InputValue};
    pub use crate::event::Event;
    pub use crate::context::Context;
    pub use crate::websockets::WebsocketPipe;
    pub use crate::node_builder::{NodeBuilder, ElementBuilder};
    pub use crate::blob::Blob;
    pub use crate::runtime::{Runtime, RuntimeControl};
}

pub use crate::component::{Component, Updated};
pub use crate::runtime::{Runtime, RuntimeControl};
pub use crate::websockets::WebsocketPipe;

/// Type to produce unique IDs within the process.
///
/// Ids may be generated from different threads and are guaranteed
/// to be unique. They may be used to reference data.
///
/// Ids are best passed by value, as they implement `Copy` and are
/// 8-bytes long.
///
/// ```
/// # use greenhorn::Id;
/// let id_a = Id::new();
/// let id_b = Id::new();
/// assert_ne!(id_a, id_b);
/// ```
/// Ids may also be considered empty, in which case the underlying data is 0.
/// Usage of this type should be minimized. It is mostly intended for synchronizing
/// state between remote (the web view) and the server side.
///
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Id {
    id: u64,
}

static COUNTER: AtomicU64 = AtomicU64::new(1);

impl Id {
    pub fn new() -> Id {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        Id { id }
    }

    /// Initialize an `Id` from underlying data.
    pub fn new_from_data(data: u64) -> Self {
        Id { id: data }
    }

    /// Returns whether the Id is empty.
    pub fn new_empty() -> Self {
        Self { id: 0 }
    }

    /// Returns the underlying data
    pub fn data(self) -> u64 {
        self.id
    }

    /// Returns whether the Id is considered empty
    /// i.e. the underlying data is 0.
    pub fn is_empty(self) -> bool {
        self.id == 0
    }
}

impl Default for Id {
    fn default() -> Self {
        Self::new()
    }
}

impl From<u64> for Id {
    fn from(id: u64) -> Self {
        Id { id }
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Id({})", self.id)
    }
}

impl PartialEq for Id {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Id {}

impl Hash for Id {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

/// Marks a type as render-able to the DOM.
///
/// A `Render` type also defines a `Message` type, which defines a type being emitted
/// by the [`Nodes`](node/enum.Node.html) it emits.
/// Implementing this trait is the minimum requirement to create
/// a [Component](component/struct.Component.html).
///
/// # Example
///
/// ```
/// # use greenhorn::Render;
/// # use greenhorn::node::Node;
/// # use greenhorn::dom::DomEvent;
/// # use greenhorn::html;
///
/// struct Button {
///     text: String,
/// }
///
/// enum ButtonMessage {
///     Clicked(DomEvent),
///     KeyDown(DomEvent),
/// }
///
/// impl Render for Button {
///     type Message = ButtonMessage;
///
///     fn render(&self) -> Node<Self::Message> {
///         html!(
///             <button type="button" @keydown={ButtonMessage::KeyDown} @mousedown={ButtonMessage::Clicked}>
///                 {&self.text}
///             </>
///         ).into()
///     }
/// }
/// ```
///
pub trait Render {
    /// Defines a type which is emitted when capturing DOM events or component events
    ///
    /// Typically this is an enum.
    type Message: 'static + Send;

    /// Renders k
    fn render(&self) -> Node<Self::Message>;
}

/// Marks a type as render-able as well as update-able
///
/// The update method should modify the state of the object and return an
/// [`Updated`](component/struct.Updated.html) object marking whether the update should
/// requires a re-render of the DOM.
///
/// At the same time an optional `mount()` function may be provided to allow a component to
/// hook into the startup cycle of a component or application.
pub trait App: Render {

    /// Modify the state of this object based on the received `Message`.
    /// Returns an [`Updated`](component/struct.Updated.html) which should mark whether this
    /// component is to be re-rendered.
    ///
    /// Each component is required to dispatch messages its child components if applicable.
    fn update(&mut self, msg: Self::Message, ctx: Context<Self::Message>) -> Updated;

    /// Shall be called upon application startup.
    /// A parent component is required to call this function of all child components.
    fn mount(&mut self, _ctx: Context<Self::Message>) {}
}


use proc_macro_hack::proc_macro_hack;

/// Proc macro to generate HTML [Nodes](struct.Node.html) implementing a JSX like syntax.
///
/// ```
/// # use greenhorn::html;
/// # use greenhorn::prelude::{Render, Node};
/// # struct MyComponent {}
///
/// impl Render for MyComponent {
///     type Message = ();
///
///     fn render(&self) -> Node<Self::Message> {
///         html!(
///             <div .class-name #html-id my-attribute="value">
///                 {"text"}
///             </>
///         ).into()
///     }
/// }
/// ```
///
#[proc_macro_hack(support_nested)]
pub use html_macro::html;

/// Proc macro to generate SVG [Nodes](struct.Node.html) implementing a JSX like syntax.
///
/// ```
/// # use greenhorn::svg;
/// # use greenhorn::prelude::{Render, Node};
/// # struct MyComponent {}
///
/// impl Render for MyComponent {
///     type Message = ();
///
///     fn render(&self) -> Node<Self::Message> {
///         svg!(
///             <g>
///                 <line x1={-1.0} x2={-1.0} y1={-1.0} y2={1.0} />
///             </>
///         ).into()
///     }
/// }
/// ```
///
#[proc_macro_hack(support_nested)]
pub use html_macro::svg;
use crate::context::Context;
use crate::node::Node;

