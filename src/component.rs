//! This module defines components, which support a fine-grained `render()` cycle.
//!
//! Types which are [Render](../trait.Render.html) or [App](../trait.App.html) can we wrapped into
//! a [`Component<_>`](struct.Component.html) to support fine-grained `render()` cycles.
//! During the `update()` components signal the runtime (using [Updated](struct.Updated.html))
//! whether re-rendering of their associated DOMs is required.
//! The runtime tracks all components and selectively calls their `render()` functions, thus
//! reducing the size of the DOM that needs to be diffed in each cycle.
//!
//! Note that `Component<_>` instances also require their contents to be `Send` to support parallel
//! rendering.
//!
//! # Example
//!
//! ```
//! # use greenhorn::event::Event;
//! # use greenhorn::{Render, Component, App, Updated};
//! # use greenhorn::node::Node;
//! # use greenhorn::dom::DomEvent;
//! # use greenhorn::context::Context;
//! # use greenhorn::html;
//! #
//! // --------- This would be typically in its own module or even crate ------------
//!
//! struct Button {
//!     text: String,
//!     clicked: Event<()>,
//! }
//!
//! impl Button {
//!     fn new(text: &str) -> Self {
//!         Self { text: text.to_string(), clicked: Event::new() }
//!     }
//! }
//!
//! enum ButtonMessage {
//!     MouseDown(DomEvent),
//!     KeyDown(DomEvent),
//! }
//!
//! impl Render for Button {
//!     type Message = ButtonMessage;
//!
//!     fn render(&self) -> Node<Self::Message> {
//!         html!(
//!             <button type="button"
//!                     @keydown={ButtonMessage::KeyDown}
//!                     @mousedown={ButtonMessage::MouseDown}>
//!                 {&self.text}
//!             </>
//!         ).into()
//!     }
//! }
//!
//! impl App for Button {
//!     fn update(&mut self, msg: Self::Message, ctx: Context<Self::Message>) -> Updated {
//!         match msg {
//!             ButtonMessage::MouseDown(evt) => {
//!                 if evt.into_mouse().unwrap().button == 1 {
//!                     ctx.emit(&self.clicked, ());
//!                 }
//!             },
//!             ButtonMessage::KeyDown(evt) => {
//!                 if evt.into_keyboard().unwrap().key == "Enter" {
//!                     ctx.emit(&self.clicked, ());
//!                 }
//!             }
//!         }
//!         Updated::no()
//!     }
//! }
//!
//! // --------- Probably in a different file such as app.rs ------------
//!
//! struct MyApp {
//!     ok_button: Component<Button>,
//!     cancel_button: Component<Button>,
//! }
//!
//! impl MyApp {
//!     fn new() -> Self {
//!         Self {
//!             ok_button: Component::new(Button::new("OK")),
//!             cancel_button: Component::new(Button::new("Cancel")),
//!         }
//!     }
//! }
//!
//! enum MyMsg {
//!     OkButton(ButtonMessage),
//!     CancelButton(ButtonMessage),
//!     OkClicked,
//!     CancelClicked,
//! }
//!
//! impl Render for MyApp {
//!     type Message = MyMsg;
//!
//!     fn render(&self) -> Node<Self::Message> {
//!         html!(
//!             <div #my-buttons>
//!                 {self.ok_button.mount().map(MyMsg::OkButton)}
//!                 {self.ok_button.map(|x| x.clicked.subscribe(|_| MyMsg::OkClicked))}
//!                 {self.cancel_button.mount().map(MyMsg::CancelButton)}
//!                 {self.cancel_button.map(|x| x.clicked.subscribe(|_| MyMsg::CancelClicked))}
//!             </>
//!         ).into()
//!     }
//! }
//!
//! impl App for MyApp {
//!     fn update(&mut self, msg: Self::Message, ctx: Context<Self::Message>) -> Updated {
//!         match msg {
//!             MyMsg::OkButton(msg) => {
//!                 self.ok_button.update(msg, ctx.map(MyMsg::OkButton))
//!             }
//!             MyMsg::CancelButton(msg) => {
//!                 self.cancel_button.update(msg, ctx.map(MyMsg::CancelButton))
//!             }
//!             MyMsg::OkClicked => {
//!                 println!("Ok Clicked!");
//!                 Updated::no()
//!             }
//!             MyMsg::CancelClicked => {
//!                 println!("Cancel Clicked!");
//!                 Updated::no()
//!             }
//!         }
//!     }
//! }
//! ```

use std::ops::DerefMut;
use crate::context::Context;
use crate::{Id, Render, App};
use std::fmt::{Debug, Formatter, Error};
use crate::node::{Node, NodeItems};
use std::collections::HashSet;
use std::collections::hash_map::RandomState;
use std::sync::{Arc, Mutex, MutexGuard};


/// Allows the `update()` cycle of an application or component to signal the runtime what portion
/// of the DOM requires re-rendering.
///
/// Usually components return either `Updated::yes()` or `Updated::no()`.
/// If a component has child components, it should must use either `Updated::merge()` or
/// `Update::combine()` to combine multiple `Updated` objects.
pub struct Updated {
    pub(crate) should_render: bool,
    pub(crate) components_render: Option<Vec<Id>>,
}

impl Updated {
    /// Alias for `Updated::no()`. Creates a new `Updated` object which signal that *no* re-render
    /// is required.
    pub fn new() -> Updated {
        Updated {
            should_render: false,
            components_render: None,
        }
    }

    /// Creates a new `Updated` object which signal that a re-render is *required*.
    pub fn yes() -> Updated {
        Updated {
            should_render: true,
            components_render: None
        }
    }
    /// Creates a new `Updated` object which signal that *no* re-render is required.
    pub fn no() -> Updated {
        Updated {
            should_render: false,
            components_render: None
        }
    }

    /// Marks a component as invalid and thus signals that a re-render is required.
    pub fn invalidate<T: Render + Send>(mut self, component: &Component<T>) -> Self {
        if let Some(ref mut ids) = self.components_render {
            ids.push(component.id)
        } else {
            self.components_render = Some(vec![component.id])
        }
        self
    }

    /// Merge another `Updated` object into this object combining the invalidated component.
    ///
    /// Equivalent to `Updated::combine(a, b)`
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

    /// Combine two `Updated` objects by combining the invalidated component.
    ///
    /// Equivalent to `a.merge(b)`.
    pub fn combine(first: Updated, second: Updated) -> Updated {
        let mut first = first;
        first.merge(second);
        first
    }

    /// Returns `true` if no component is scheduled to be re-rendered.
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

/// A `Component` wraps a `Render` or `App` type to allow the runtime for fine-grained `render()`
/// calls. This avoids re-rendering the whole DOM in each cycle.
///
/// Components can be mounted to a DOM using the `Component::mount()` function.
/// Each component has an assigned `id()` which allows it to be identified by the runtime.
/// Invalidated components are recorded with their `id()` in the `Updated` type.
///
/// The underlying data type can be accessed using the `lock()`, `map()` or `transmute()` functions.
///
/// For a more detailed example, refer to the [module level documentation](index.html).
///
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

    /// Creates a new `Component` by taking ownership of the underlying type.
    pub fn new(inner: T) -> Self {
        Self {
            id: Id::new(),
            comp: Arc::new(Mutex::new(inner)),
        }
    }

    /// Locks the underlying data for modification and returns a reference to it
    pub fn lock(&self) -> MutexGuard<T> {
        self.comp.lock().unwrap()
    }

    /// Returns the unique `Id` associated with this component
    ///
    /// The `id()` is used to identify this component in the DOM hierarchy.
    /// THe executing runtime understands, based on the result of the `update()` cycle,
    /// which yields and [Updated](struct.Updated.html) object, which components require
    /// re-rendering.
    pub fn id(&self) -> Id {
        self.id
    }

    /// Calls the `render()` function of the underlying `Render` object.
    pub fn render(&self) -> Node<T::Message> {
        self.lock().render()
    }

    /// Applies a function to the underlying data and returns it's result. The function
    /// receives an immutable reference.
    pub fn map<R, F: Fn(&T) -> R>(&self, fun: F) -> R {
        let data = self.lock();
        fun(&data)
    }

    /// Applies a function to the underlying data and returns it's result. The function
    /// receives an mutable reference.
    pub fn transmute<R, F: FnOnce(&mut T) -> R>(&self, fun: F) -> R {
        let mut data = self.lock();
        fun(data.deref_mut())
    }

    /// Mounts the component to the DOM.
    ///
    /// Usually, this function in chained with a `.map()` call on the result to map it to
    /// the `Node` with the appropriate message type.
    ///
    /// ## Example
    ///
    /// ```
    /// # use greenhorn::{Component, Render};
    /// # use greenhorn::node::Node;
    /// # use greenhorn::html;
    /// #
    /// # struct Button {
    /// #     text: String
    /// # }
    /// #
    /// # enum ButtonMsg {
    /// #     Something
    /// # }
    /// #
    /// # impl Render for Button {
    /// #    type Message = ButtonMsg;
    /// #
    /// #     fn render(&self) -> Node<Self::Message> {
    /// #         unimplemented!()
    /// #    }
    /// #
    /// # }
    /// #
    /// struct MyApp {
    ///     ok_button: Component<Button>,
    /// }
    ///
    /// enum MyMsg {
    ///     OkButton(ButtonMsg)
    /// }
    ///
    /// impl Render for MyApp {
    ///     type Message = MyMsg;
    ///
    ///     fn render(&self) -> Node<MyMsg> {
    ///         html!(
    ///             <div>
    ///                 {self.ok_button.mount().map(MyMsg::OkButton)}
    ///             </>
    ///         ).into()
    ///     }
    /// }
    /// ```
    pub fn mount(&self) -> Node<T::Message> {
        Node(NodeItems::Component(ComponentContainer::new(self)))
    }
}
/// If the underlying type is also `App` the `Component` also provides access to the
/// `App::update()` and `App::mount()` function.
impl<T: 'static + App + Send> Component<T> {

    /// If the underlying type is also `App`, this function provides direct access
    /// to the `update()` function of the underlying component.
    pub fn update(&mut self, msg: T::Message, ctx: Context<T::Message>) -> Updated {
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

    /// If the underlying type is also `App`, this function provides direct access
    /// to the `mount()` function of the underlying component.
    pub fn on_mount(&mut self, ctx: Context<T::Message>) {
        let mut borrow = self.lock();
        let data = borrow.deref_mut();
        data.mount(ctx);
    }
}

/// wraps a shared `ComponentMap` trait object to improve the internal API
pub(crate) struct ComponentContainer<T: 'static + Send> {
    pub(crate) inner: Arc<Mutex<dyn ComponentMap<T>>>,
}

impl<T: 'static + Send> ComponentContainer<T> {
    /// Create a new ComponentContainer based on a component
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

/// This trait defines the interface for the runtime to access the wrapped functionality.
///
/// Used to implement type erasure.
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

/// Maps a compenent from one message type to another
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

