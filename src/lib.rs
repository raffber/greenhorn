#![allow(dead_code)]
#![recursion_limit = "256"]

//!
//! # Greenhorn - API Documentation
//!
//! Greenhorn is a rust library for building desktop applications with web technologies in (almost)
//! pure rust.
//!
//! This is accomplished by separating the application into a server-side process
//! (the backend) and web view implemented in javascript (the frontend).
//! While most HTML-based desktop applications leave state synchronization up to the
//! application logic, this library synchronizes its state at DOM-level.
//! Thus, the user may implement the application logic purely in the backend using rust.
//! This facilitates the integration of a desktop GUI with system
//! services and simplifies application development considerably.
//!
//! ## Features
//!
//! * Elm-like architecture but also supports components
//! * Components support fine-grained update/render cycle
//! * Components are owned by the application state and may interact with each other using events
//! * Macros to write SVG and HTML in-line with Rust code
//! * Most tasks can be accomplished using pure-rust. If required, injecting and calling js is supported.
//! * Built-in performance metrics
//! * Spawning system dialogs
//! * Optionally deploy using [web_view](https://github.com/Boscop/web-view) and [tinyfiledialogs-rs](https://github.com/jdm/tinyfiledialogs-rs)
//!
//! ## Example
//!
//! ```
//! use greenhorn::prelude::*;
//! use greenhorn::html;
//!
//! struct MyApp {
//!     text: String,
//! }
//!
//! enum MyMsg {
//!     Clicked(DomEvent),
//!     KeyDown(DomEvent),
//! }
//!
//! impl Render for MyApp {
//!     type Message = MyMsg;
//!
//!     fn render(&self) -> Node<Self::Message> {
//!         html!(
//!             <div #my-app>
//!                 <button type="button"
//!                         @keydown={MyMsg::KeyDown}
//!                         @mousedown={MyMsg::Clicked}>
//!                     {&self.text}
//!                 </>
//!             </>
//!         ).into()
//!     }
//! }
//!
//! impl App for MyApp {
//!     fn update(&mut self,msg: Self::Message,ctx: Context<Self::Message>) -> Updated {
//!         match msg {
//!             MyMsg::Clicked(evt) => self.text = "Button clicked!".into(),
//!             MyMsg::KeyDown(evt) => self.text = "Button keypress!".into()
//!         }
//!         Updated::yes()
//!     }
//!
//! }
//!
//! ```
//!

use serde::{Deserialize, Serialize};
use std::cmp::Eq;
use std::convert::From;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};

/// Components allow code reuse and support modularization
pub mod component;

/// Maps javascript DOM types to rust types. In particular DOM events.
pub mod dom;

/// Defines events which allow components to interact with each other
pub mod event;

/// Supports interaction between the applications `update()` cycle and system services
pub mod context;

/// Rust API for building DOM nodes. Alternative to the `html!()` and `svg!()` macros
pub mod node_builder;

/// Defines the interface for interacting between frontend and backend
pub mod pipe;

/// Implments the `Runtime` type, which executes the render/update cycle of the application
pub mod runtime;

/// Supports spawning tasks running on the frontend. Experimental, might be removed in the future.
pub mod service;

/// Virtual DOM implementation with diffing and patch generation
mod vdom;

/// Implements a `Pipe` using websockets
pub mod websockets;

/// Defines `Node<T>` type for building DOMs in pure rust
pub mod node;

/// Implements listeners of DOM events on DOM elements
pub mod listener;

/// Supports syncing binary data from backend to frontend. Useful for images, media files, ...
pub mod blob;

/// TODO: make private
pub mod element;

/// Provides a set of built-in and commonly used components
pub mod components;

/// Allows spawning native dialogs such as file-open dialogs and retrieving their results.
pub mod dialog;

/// Exposes a built-in set of commonly used services
pub mod services;

/// Supports dynamic binding of components. Experimental, might be removed in the future.
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

