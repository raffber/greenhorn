use crate::dom::DomEvent;
use crate::Id;
use serde_json::Value as JsonValue;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

pub(crate) struct Rpc<T> {
    pub(crate) node_id: Id,
    pub(crate) fun: Arc<Mutex<dyn Fn(JsonValue) -> T + Send>>,
}

impl<T> Clone for Rpc<T> {
    fn clone(&self) -> Self {
        Self {
            node_id: self.node_id,
            fun: self.fun.clone(),
        }
    }
}

impl<T: 'static> Rpc<T> {
    pub(crate) fn map<U: 'static>(
        self,
        fun: Arc<Mutex<dyn 'static + Send + Fn(T) -> U>>,
    ) -> Rpc<U> {
        let self_fun = self.fun;
        let new_fun = move |e: JsonValue| {
            let unlocked_fun = self_fun.lock().unwrap();
            let inner_result: T = (unlocked_fun)(e);
            let ret: U = (fun.lock().unwrap())(inner_result);
            ret
        };
        let new_fun: Arc<Mutex<dyn Fn(JsonValue) -> U + Send>> =
            Arc::new(Mutex::new(Box::new(new_fun)));
        Rpc {
            node_id: self.node_id,
            fun: new_fun,
        }
    }

    pub(crate) fn call(&self, e: JsonValue) -> T {
        (self.fun.lock().unwrap())(e)
    }
}

pub(crate) struct Listener<T> {
    pub(crate) event_name: String,
    pub(crate) node_id: Id,
    pub(crate) fun: Arc<Mutex<dyn Fn(DomEvent) -> T + Send>>,
    pub(crate) no_propagate: bool,
    pub(crate) prevent_default: bool,
}

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
    pub(crate) fn map<U: 'static>(
        self,
        fun: Arc<Mutex<dyn 'static + Send + Fn(T) -> U>>,
    ) -> Listener<U> {
        let self_fun = self.fun;
        let new_fun = move |e: DomEvent| {
            let unlocked_fun = self_fun.lock().unwrap();
            let inner_result: T = (unlocked_fun)(e);
            let ret: U = (fun.lock().unwrap())(inner_result);
            ret
        };
        let new_fun: Arc<Mutex<dyn Fn(DomEvent) -> U + Send>> =
            Arc::new(Mutex::new(Box::new(new_fun)));
        Listener {
            event_name: self.event_name,
            node_id: self.node_id,
            fun: new_fun,
            no_propagate: self.no_propagate,
            prevent_default: self.prevent_default,
        }
    }

    pub fn call(&self, e: DomEvent) -> T {
        (self.fun.lock().unwrap())(e)
    }
}

#[derive(Eq, Debug, Clone)]
pub(crate) struct ListenerKey {
    hash: u64,
}

impl Hash for ListenerKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

impl ListenerKey {
    pub(crate) fn new<M: 'static + Send>(listener: &Listener<M>) -> Self {
        let mut hasher = DefaultHasher::new();
        hasher.write_u64(listener.node_id.id);
        hasher.write(listener.event_name.as_bytes());
        let hash = hasher.finish();
        Self { hash }
    }

    pub(crate) fn from_raw(id: Id, name: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        hasher.write_u64(id.id);
        hasher.write(name.as_bytes());
        let hash = hasher.finish();
        Self { hash }
    }
}

impl PartialEq for ListenerKey {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::{BaseEvent, InputValue};

    #[derive(Debug)]
    enum MsgInner {
        Event(DomEvent),
    }

    #[derive(Debug)]
    enum MsgOuter {
        Inner(MsgInner),
    }

    #[test]
    fn map_listener() {
        let listener = Listener {
            event_name: "".to_string(),
            node_id: Id::new(),
            fun: Arc::new(Mutex::new(Box::new(MsgInner::Event))),
            no_propagate: false,
            prevent_default: false,
        };
        let mapped = listener.map(Arc::new(Mutex::new(Box::new(MsgOuter::Inner))));
        let evt = BaseEvent {
            target: Default::default(),
            event_name: "foo".to_string(),
            target_value: InputValue::NoValue,
        };
        let evt = DomEvent::Base(evt);
        let msg = mapped.call(evt);
        assert_matches::assert_matches!(msg, MsgOuter::Inner(MsgInner::Event(_)))
    }
}
