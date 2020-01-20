use std::sync::{Arc, Mutex};
use crate::Id;
use crate::dom_event::DomEvent;

pub struct Listener<T> {
    pub event_name: String,
    pub node_id: Id,
    pub fun: Arc<Mutex<Box<dyn Fn(DomEvent) -> T + Send>>>,
    pub no_propagate: bool,
    pub prevent_default: bool,
}

// TODO: derive(Clone) failed for some reason?!
impl<T> Clone for Listener<T> {
    fn clone(&self) -> Self {
        Listener {
            event_name: self.event_name.clone(),
            node_id: self.node_id,
            fun: self.fun.clone(),
            no_propagate: self.no_propagate,
            prevent_default: self.prevent_default,
        }
    }
}

impl<T: 'static> Listener<T> {
    pub fn map<U: 'static>(self, fun: Arc<dyn Fn(T) -> U>) -> Listener<U> {
        let self_fun = self.fun;
        Listener {
            event_name: self.event_name,
            node_id: self.node_id,
            fun: Arc::new(move |e| fun((self_fun)(e))),
            no_propagate: self.no_propagate,
            prevent_default: self.prevent_default,
        }
    }

    pub fn call(&self, e: DomEvent) -> T {
        (self.fun)(e)
    }
}

