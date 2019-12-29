use std::cell::{Ref, RefCell, RefMut};
use std::ops::Deref;
use std::rc::Rc;

use crate::mailbox::Mailbox;
use crate::node_builder::NodeBuilder;
use crate::Id;
use std::fmt::{Debug, Formatter, Error};
use crate::node::Node;

pub struct Updated {
    pub(crate) should_render: bool,
    pub(crate) components_render: Option<Vec<Id>>,
}

impl Updated {
    fn new() -> Updated {
        Updated {
            should_render: false,
            components_render: None,
        }
    }

    fn render(mut self) -> Self {
        self.should_render = true;
        self
    }

    fn invalidate(mut self, id: Id) -> Self {
        if let Some(ref mut ids) = self.components_render {
            ids.push(id)
        } else {
            self.components_render = Some(vec![id])
        }
        self
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

pub struct Component<T: Render> {
    id: Id,
    comp: Rc<RefCell<T>>,
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
            comp: Rc::new(RefCell::new(inner)),
        }
    }

    pub fn borrow_mut(&mut self) -> RefMut<T> {
        self.comp.borrow_mut()
    }

    pub fn borrow(&self) -> Ref<T> {
        self.comp.borrow()
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn render(&self) -> Node<T::Message> {
        self.comp.deref().borrow().render()
    }

    pub fn map<R, F: Fn(&T) -> R>(&self, fun: F) -> R {
        let data = self.comp.deref().borrow();
        fun(&data)
    }

    pub fn update<R, F: FnOnce(&mut T) -> R>(&mut self, fun: F) -> R {
        let mut borrow = self.comp.deref().borrow_mut();
        let data = &mut borrow;
        fun(data)
    }
}

pub trait Render {
    type Message: 'static + Send;
    fn render(&self) -> Node<Self::Message>;

    fn html(&self) -> NodeBuilder<Self::Message> {
        NodeBuilder::new()
    }

    fn svg(&self) -> NodeBuilder<Self::Message> {
        NodeBuilder::new_with_ns("http://www.w3.org/2000/svg")
    }
}

pub trait App: Render {
    fn update(&mut self, msg: Self::Message, mailbox: Mailbox<Self::Message>) -> Updated;
    fn mount(&mut self, _mailbox: Mailbox<Self::Message>) {
    }
}
