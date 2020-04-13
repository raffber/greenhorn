use crate::{App, Id};
use crate::event::Subscription;
use std::collections::HashMap;
use crate::listener::{Listener, ListenerKey, Rpc};
use crate::runtime::RenderResult;

#[cfg(test)]
use crate::vdom::VNode;

pub(crate) struct Frame<A: App> {
    pub(crate) rendered: RenderResult<A>,
    pub(crate) translations: HashMap<Id, Id>, // new -> old
}

impl<A: App> Frame<A> {
    pub(crate) fn new(rendered: RenderResult<A>, translations: HashMap<Id, Id>) -> Self {
        Self { rendered, translations }
    }

    #[cfg(test)]
    pub(crate) fn new_from_vnode(vdom: VNode) -> Self {
        Self {
            rendered: RenderResult::new_from_vnode(vdom),
            translations: Default::default()
        }
    }
}


pub(crate) struct RenderedState<A: App> {
    subscriptions: HashMap<Id, Subscription<A::Message>>,
    listeners: HashMap<ListenerKey, Listener<A::Message>>,
    translations: HashMap<Id, Id>, // old -> new
    rpcs: HashMap<Id, Rpc<A::Message>>,
}


impl<A: App> RenderedState<A> {
    pub(crate) fn new() -> Self {
        Self {
            subscriptions: Default::default(),
            listeners: Default::default(),
            translations: Default::default(),
            rpcs: Default::default()
        }
    }

    pub(crate) fn get_rpc(&self, target: Id) -> Option<&Rpc<A::Message>> {
        self.rpcs.get(&target)
    }

    pub(crate) fn get_listener(&self, target: Id, name: &str) -> Option<&Listener<A::Message>>{
        let target = self.translations.get(&target).unwrap_or(&target);
        let key = ListenerKey::from_raw(*target, &name);
        self.listeners.get(&key)
    }

    pub(crate) fn get_subscription(&self, event_id: Id) -> Option<&Subscription<A::Message>> {
        self.subscriptions.get(&event_id)
    }

    pub(crate) fn apply(&mut self, frame: &Frame<A>) {
        self.listeners = frame.rendered.listeners.clone();
        self.rpcs = frame.rendered.rpcs.clone();
        self.subscriptions = frame.rendered.subscriptions.clone();
        self.translations.clear();
        for (new, old) in &frame.translations {
            self.translations.insert(*old, *new);
        }
    }
}
