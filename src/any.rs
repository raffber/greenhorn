use crate::{App, Render, Updated};
use std::any::Any;
use crate::node::Node;
use crate::mailbox::Mailbox;

pub struct AnyApp {
    inner: Box<dyn App<Message=Box<dyn Any + Send>>>,
}

impl AnyApp {
    pub fn from_app<T: 'static + App<Message=M>, M: Any + Send + 'static>(app: T) -> AnyApp {
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
    fn update(&mut self, msg: Self::Message, mailbox: Mailbox<Self::Message>) -> Updated {
        self.inner.update(msg, mailbox)
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
    fn update(&mut self, msg: Self::Message, mailbox: Mailbox<Self::Message>) -> Updated {
        let new_msg = *msg.downcast().unwrap();
        self.inner.update(new_msg, mailbox.map(|x| Box::new(x) as Box<dyn Any + Send + 'static>))
    }
}
