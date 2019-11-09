use std::borrow::Borrow;
use std::rc::Rc;
use std::cell::RefCell;
use crate::component::Node::Dummy;
use std::sync::Arc;

struct Update {}

struct Id {}

pub enum NodeKind {
    Text,
    Component,
    Element,
}


trait NodeMap<T> {
    fn listeners(&self) -> Vec<Listener<T>>;
}

struct NodeMapImpl<T, U> {
    fun: Arc<dyn Fn(T) -> U>,
    inner: Node<T>,
}

impl<T: 'static, U: 'static> NodeMapImpl<T, U> {
    fn new<F: 'static + Fn(T) -> U>(fun: F, inner: Node<T>) -> Box<dyn NodeMap<U>> {
        Box::new(NodeMapImpl {
            fun: Arc::new(fun),
            inner
        })
    }
}

impl<T: 'static, U: 'static> NodeMap<U> for NodeMapImpl<T, U> {
    fn listeners(&self) -> Vec<Listener<U>> {
        self.inner.listeners().drain(..).map(|x| x.map(self.fun.clone())).collect()
    }
}

enum Node<T> {
    Map(Box<dyn NodeMap<T>>),
    Dummy(std::marker::PhantomData<T>)
}


impl<T: 'static> Node<T> {
    pub fn map<U: 'static, F: 'static + Fn(T) -> U>(self, fun: F) -> Node<U> {
        Node::Map(NodeMapImpl::new(fun, self))
    }

    fn listeners(&self) -> Vec<Listener<T>> {
        unimplemented!()

    }
}

enum Event {
    RawEvent()
}

struct Listener<T> {
    pub name: String,
    pub node_id: Id,
    pub fun: Box<dyn Fn(Event) -> T>
}

impl<T: 'static> Listener<T> {
    fn map<U: 'static>(self, fun: Arc<dyn Fn(T) -> U>) -> Listener<U> {
        let self_fun = self.fun;
        Listener {
            name: self.name,
            node_id: self.node_id,
            fun: Box::new(move |e| {
                fun( (self_fun)(e))
            })
        }
    }

    fn call(&self, e: Event) -> T {
        (self.fun)(e)
    }
}


struct Component<T: Render> {
    comp: Rc<RefCell<T>>
}

impl<T: Render> Component<T> {

}

trait Render {
    type Message;
    fn render(&self) -> Node<Self::Message>;
}

trait App : Render {
    fn update(&mut self, msg: Self::Message) -> Update;
}
