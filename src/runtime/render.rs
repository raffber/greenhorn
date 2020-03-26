use crate::vdom::VNode;
use crate::node::{Node, ComponentContainer, ComponentMap, Blob};

use crate::{App, Id};
use crate::listener::Listener;
use crate::event::Subscription;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use crate::runtime::metrics::Metrics;
use crate::runtime::component::RenderedComponent;

// TODO: currently an event cannot be subscribed to multiple times
// since we store the event_id as the key to find a single subscription
// however, we should use a subscription list as value

pub(crate) enum ResultItem<A: App> {
    Listener( Listener<A::Message> ),
    Subscription( Id, Subscription<A::Message> ),
    Component( ComponentContainer<A::Message> ),
    Blob( Blob )
}


pub(crate) struct RenderResult<A: App> {
    listeners: HashMap<ListenerKey, Listener<A::Message>>,
    pub(crate) subscriptions: HashMap<Id, Subscription<A::Message>>,
    pub(crate) blobs: HashMap<Id, Blob>,
    components: HashMap<Id, Arc<RenderedComponent<A>>>,
    pub(crate) root_components: HashSet<Id>,
    pub(crate) root: Arc<VNode>,
}

impl<A: App> RenderResult<A> {
    pub(crate) fn from_vnode(root: VNode) -> Self {
        Self {
            listeners: Default::default(),
            subscriptions: Default::default(),
            blobs: Default::default(),
            components: Default::default(),
            root_components: Default::default(),
            root: Arc::new(root)
        }
    }

    pub(crate) fn from_root(root_rendered: Node<A::Message>, _metrics: &mut Metrics) -> Self {
        let mut result = Vec::new();
        let vdom = RenderedComponent::<A>::render_recursive(root_rendered, &mut result)
            .expect("Root produced an empty DOM");

        let mut ret = Self {
            listeners: Default::default(),
            subscriptions: Default::default(),
            blobs: Default::default(),
            components: HashMap::default(),
            root_components: HashSet::new(),
            root: Arc::new(vdom),
        };

        for item in result.drain(..) {
            match item {
                ResultItem::Listener(listener) => {
                    ret.listeners.insert(ListenerKey::new(&listener), listener);
                },
                ResultItem::Subscription(id, subscription) => {
                    ret.subscriptions.insert(id, subscription);
                },
                ResultItem::Component(comp) => {
                    ret.root_components.insert(comp.id());
                    ret.render_component(comp);
                }
                ResultItem::Blob(blob) => {
                    ret.blobs.insert(blob.id(), blob);
                }
            }
        }
        ret
    }

    fn render_component(&mut self, comp: ComponentContainer<A::Message>) {
        let id = comp.id();
        let (rendered, mut result) = RenderedComponent::new(comp);
        self.components.insert(id, Arc::new(rendered));

        for item in result.drain(..) {
            match item {
                ResultItem::Listener(listener) => {
                    self.listeners.insert(ListenerKey::new(&listener), listener);
                },
                ResultItem::Subscription(id, subscription) => {
                    self.subscriptions.insert(id, subscription);
                },
                ResultItem::Component(comp) => {
                    self.render_component(comp);
                }
                ResultItem::Blob(blob) => {
                    self.blobs.insert(blob.id(), blob);
                }
            }
        }
    }

    fn render_component_from_old(&mut self, old: &RenderResult<A>,
            comp: ComponentContainer<A::Message>,
            rendered: &HashSet<Id>) {
        let id = comp.id();
        if !rendered.contains(&id) && old.components.contains_key(&id) {
            let old_render = old.components.get(&id).unwrap();
            for child in old_render.children() {
                let old_comp = old.components.get(&child).unwrap();
                self.render_component_from_old(old, old_comp.component(), rendered)
            }
            for key in old_render.listeners() {
                let listener = old.listeners.get(key).unwrap();
                self.listeners.insert(key.clone(), listener.clone());
            }
            for event_id in old_render.subscriptions() {
                let subs = old.subscriptions.get(&event_id).unwrap();
                self.subscriptions.insert(*event_id, subs.clone());
            }
            for blob_id in old_render.blobs() {
                let blob = old.blobs.get(&blob_id).unwrap();
                self.blobs.insert(*blob_id, blob.clone());
            }
            self.components.insert(id, old_render.clone());
            return;
        }
        let (rendered_component, mut result) = RenderedComponent::new(comp);
        self.components.insert(id, Arc::new(rendered_component));
        for item in result.drain(..) {
            match item {
                ResultItem::Listener(listener) => {
                    self.listeners.insert(ListenerKey::new(&listener), listener);
                },
                ResultItem::Subscription(id, subscription) => {
                    self.subscriptions.insert(id, subscription);
                },
                ResultItem::Component(comp) => {
                    self.render_component_from_old(old, comp, rendered);
                },
                ResultItem::Blob(blob) => {
                    self.blobs.insert(blob.id(), blob);
                }
            }
        }
    }

    /// precondition: The root component must still be valid and not require a re-render
    pub(crate) fn from_frame(old: &Frame<A>, changes: &HashSet<Id>, _metrics: &mut Metrics) -> Self {
        let mut old = &old.rendered;
        let mut ret = Self {
            listeners: Default::default(),
            subscriptions: Default::default(),
            blobs: Default::default(),
            components: HashMap::with_capacity(old.components.len() * 2 ),
            root_components: HashSet::new(),
            root: old.root.clone(),
        };

        let root_components = old.root_components.clone(); // XXX: workaround
        for id in &root_components {
            let comp = old.components.get(id).unwrap();
            ret.render_component_from_old(&mut old, comp.component(), changes);
        }

        ret.root_components = old.root_components.clone();
        ret.root = old.root.clone();
        ret
    }

    pub(crate) fn get_component_vdom(&self, component_id: &Id) -> Option<&VNode> {
        self.components.get(component_id).map(|x| x.vdom())
    }
}

pub(crate) struct Frame<A: App> {
    pub(crate) rendered: RenderResult<A>,
    pub(crate) translations: HashMap<Id, Id>, // new -> old
}

impl<A: App> Frame<A> {
    pub(crate) fn new(rendered: RenderResult<A>, translations: HashMap<Id, Id>) -> Self {
        Self { rendered, translations }
    }

    pub(crate) fn from_vnode(vdom: VNode) -> Self {
        Self {
            rendered: RenderResult::from_vnode(vdom),
            translations: Default::default()
        }
    }
}

#[derive(Hash, Eq, Debug, Clone)]
pub(crate) struct ListenerKey {
    id: Id,
    name: String,
}

impl ListenerKey {
    pub(crate) fn new<M: 'static + Send>(listener: &Listener<M>) -> Self {
        Self {
            id: listener.node_id,
            name: listener.event_name.clone()
        }
    }

    pub(crate) fn from_raw(id: Id, name: &str) -> Self {
        Self { id, name: name.to_string() }
    }
}

impl PartialEq for ListenerKey {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.name == other.name
    }
}

pub(crate) struct RenderedState<A: App> {
    subscriptions: HashMap<Id, Subscription<A::Message>>,
    listeners: HashMap<ListenerKey, Listener<A::Message>>,
    translations: HashMap<Id, Id> // old -> new
}


impl<A: App> RenderedState<A> {
    pub(crate) fn new() -> Self {
        Self {
            subscriptions: Default::default(),
            listeners: Default::default(),
            translations: Default::default()
        }
    }

    pub(crate) fn get_listener(&self, target: &Id, name: &str) -> Option<&Listener<A::Message>>{
        let target = self.translations.get(target).unwrap_or(target);
        let key = ListenerKey::from_raw(*target, &name);
        self.listeners.get(&key)
    }

    pub(crate) fn get_subscription(&self, event_id: &Id) -> Option<&Subscription<A::Message>> {
        self.subscriptions.get(&event_id)
    }

    pub(crate) fn apply(&mut self, frame: &Frame<A>) {
        self.listeners = frame.rendered.listeners.clone();
        self.subscriptions = frame.rendered.subscriptions.clone();
        self.translations.clear();
        for (new, old) in &frame.translations {
            self.translations.insert(*old, *new);
        }
    }
}
