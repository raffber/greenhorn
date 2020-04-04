#![allow(dead_code)]
#![recursion_limit = "256"]

use serde::{Deserialize, Serialize};
use std::cmp::Eq;
use std::convert::From;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};

mod component;
mod dom_event;
mod event;
mod context;
mod node_builder;
pub mod pipe;
mod runtime;
mod service;
mod vdom;
mod websocket_pipe;
mod node;
mod listener;
mod blob;
mod element;
pub mod dialog;
pub mod services;
pub mod any;

pub mod prelude {
    pub use crate::component::{App, Component, Render, Updated};
    pub use crate::node::Node;
    pub use crate::dom_event::{KeyboardEvent, WheelEvent, MouseEvent, DomEvent, ChangeEvent, InputValue};
    pub use crate::event::Event;
    pub use crate::context::Context;
    pub use crate::runtime::{Runtime, RuntimeControl};
    pub use crate::websocket_pipe::WebsocketPipe;
    pub use crate::node_builder::{NodeBuilder, ElementBuilder};
    pub use crate::blob::Blob;
}

pub use crate::component::{App, Component, Render, Updated};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Id {
    id: u64,
}

static COUNTER: AtomicU64 = AtomicU64::new(1);

#[allow(clippy::new_without_default)]
impl Id {
    pub fn new() -> Id {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        Id { id }
    }

    pub fn from_data(data: u64) -> Self {
        Id { id: data }
    }

    pub fn data(self) -> u64 {
        self.id
    }

    pub fn empty() -> Self {
        Self { id: 0 }
    }

    pub fn is_empty(self) -> bool {
        self.id == 0
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

#[macro_export]
macro_rules! trait_alias {
    ($name:ident = $($trait_bound:tt)*) => {
        pub trait $name: $($trait_bound)* {}
        impl<T: $($trait_bound)*> $name for T {}
    }
}


use proc_macro_hack::proc_macro_hack;

#[proc_macro_hack(support_nested)]
pub use html_macro::html;

#[proc_macro_hack(support_nested)]
pub use html_macro::svg;

