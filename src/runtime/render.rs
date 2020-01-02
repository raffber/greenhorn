use crate::vdom::{Path, EventHandler, VElement, VNode};
use crate::node::{Node, ComponentContainer, ElementMap, ComponentMap};

use crate::{App, Id, Updated};
use crate::listener::Listener;
use crate::event::Subscription;
use std::collections::{HashMap, HashSet};
use crate::runtime::dag::Dag;

// TODO: currently an event cannot be subscribed to multiple times
// since we store the event_id as the key to find a single subscription
// however, we should use a subscription list as value

pub(crate) enum ResultItem<A: App> {
    Listener( Listener<A::Message> ),
    Subscription( Id, Subscription<A::Message> ),
    Component( ComponentContainer<A::Message> ),
}



struct RenderedComponent<A: App> {
    component: ComponentContainer<A::Message>,
    vdom: VNode,
    listeners: Vec<ListenerKey>,
    subscriptions: Vec<Id>,
    children: Vec<Id>,
}

impl<A: App> RenderedComponent<A> {
    fn new(comp: ComponentContainer<A::Message>) -> (Self, Vec<ResultItem<A>>) {
        let dom = comp.render();
        let mut result = Vec::new();
        let vdom = Self::render_recursive(dom, &mut result)
            .expect("Expected an actual DOM to render.");

        let mut subs = Vec::with_capacity(result.len());
        let mut listeners = Vec::with_capacity(result.len());
        let mut children = Vec::with_capacity(result.len());

        for item in &result {
            match item {
                ResultItem::Listener(listener) => {
                    let key = ListenerKey { id: listener.node_id, name: listener.event_name.clone() };
                    listeners.push(key)
                },
                ResultItem::Subscription(id, _) => {
                    subs.push(id.clone());
                },
                ResultItem::Component(comp) => {
                    children.push(comp.id())
                },
                _ => {}
            }
        }

        (Self {
            component: comp, vdom, listeners,
            subscriptions: subs, children
        }, result)
    }

    fn id(&self) -> Id {
        self.component.id()
    }

    fn render(&self) -> Node<A::Message> {
        self.component.render()
    }

    fn render_recursive(dom: Node<A::Message>, result: &mut Vec<ResultItem<A>>) -> Option<VNode> {
        match dom {
            Node::ElementMap(mut elem) => Self::render_element(&mut *elem, components),
            Node::Component(comp) => {
                result.push( ResultItem::Component(comp) );
                Some(VNode::Placeholder(comp.id()))
            }
            Node::Text(text) => Some(VNode::text(text)),
            Node::Element(mut elem) => Self::render_element(&mut elem, components),
            Node::EventSubscription(event_id, subs) => {
                result.push( ResultItem::Subscription(event_id, subs) );
                None
            }
        }
    }


    fn render_element(elem: &mut dyn ElementMap<A::Message>, result: &mut Vec<ResultItem<A>>) -> Option<VNode> {
        let mut children = Vec::new();
        for (idx, child) in elem.take_children().drain(..).enumerate() {
            path.push(idx);
            let child = Self::render_recursive(child, result);
            if let Some(child) = child {
                children.push(child);
            }
            path.pop();
        }
        let mut events = Vec::new();
        for listener in elem.take_listeners().drain(..) {
            events.push(EventHandler::from_listener(&listener));
            result.push( ResultItem::Listener(listener) );
        }
        Some(VNode::element(VElement {
            id: elem.id(),
            tag: elem.take_tag(),
            attr: elem.take_attrs(),
            events,
            children,
            namespace: elem.take_namespace(),
        }))
    }
}

struct TreeItem {
    dirty: bool,
    component_id: Id
}


pub(crate) struct RenderResult<A: App> {
    pub(crate) listeners: HashMap<ListenerKey, Listener<A::Message>>,
    pub(crate) subscriptions: HashMap<Id, Subscription<A::Message>>,
    pub(crate) components: HashMap<Id, RenderedComponent<A::Message>>,
    pub(crate) root_components: HashSet<Id>,
    pub(crate) root: VNode,
}

impl<A: App> RenderResult<A> {
    pub(crate) fn from_root(root_rendered: Node<A::Message>) -> Self {
        let mut result = Vec::new();
        let vdom = RenderedComponent::render_recursive(root_rendered, &mut result)
            .expect("Root produced an empty DOM");

        let mut ret = Self {
            listeners: Default::default(),
            subscriptions: Default::default(),
            components: HashMap::default(),
            root_components: HashSet::new(),
            root: vdom,
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
            }
        }
        ret
    }

    fn render_component(&mut self, comp: ComponentContainer<A::Message>) {
        let (rendered, mut result) = RenderedComponent::new(comp);
        self.components.insert(comp.id(), rendered);

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
            }
        }
    }

    fn render_component_from_old(&mut self, old: &mut RenderResult<A>,
                                 comp: ComponentContainer<A::Message>,
                                 changes: &HashSet<Id>) {
        let id = comp.id();
        if !changes.contains(&id) && old_components.contains_key(&id) {
            let old_render = old.components.remove(&id).unwrap();
            self.components.insert(id, old_render);
            for child in comp.children {
                let old_comp = old.components.remove(&child).unwrap();
                self.render_component_from_old(old_components, old_comp.component.clone(), changes)
            }
            for key in old_render.listeners {
                let (_, listener) = old.listeners.remove(&key).unwrap();
                self.listeners.insert(key, listener);
            }
            for event_id in old_render.subscriptions {
                let (_, subs) = old.subscriptions.remove(&key).unwrap();
                self.subscriptions.insert(event_id, subs);
            }
            return;
        }
        let (rendered, mut result) = RenderedComponent::new(comp);
        self.components.insert(id, rendered);
        for item in result.drain(..) {
            match item {
                ResultItem::Listener(listener) => {
                    self.listeners.insert(ListenerKey::new(&listener), listener);
                },
                ResultItem::Subscription(id, subscription) => {
                    self.subscriptions.insert(id, subscription);
                },
                ResultItem::Component(comp) => {
                    self.render_component_from_old(old_components, comp, changes);
                },
            }
        }
    }

    /// precondition: The root component must still be valid and not require a re-render
    pub(crate) fn from_frame(old: Frame<A>, changes: Updated) -> Self {
        let mut old = old.rendered;
        let changes : HashSet<Id> = changes.into();
        let mut ret = Self {
            listeners: Default::default(),
            subscriptions: Default::default(),
            components: HashMap::with_capacity(old.components.len() * 2 ),
            root_components: HashSet::new(),
            root: VNode::Placeholder(Id::empty()),
        };

        for id in &old.root_components {
            let comp = old.components.remove(id).unwrap();
            ret.render_component_from_old(&old, comp.component, &changes);
        }

        ret.root_components = old.root_components;
        ret.root = old.root;
        ret
    }
}

pub(crate) struct Frame<A: App> {
    rendered: RenderResult<A>,
    translations: HashMap<Id, Id>,
}

impl<A: App> Frame<A> {
    pub(crate) fn new(rendered: RenderResult<A>, trans: &Vec<(Id,Id)>) -> Self {
        let mut translations = HashMap::new();
        for (k, v) in trans {
            translations.insert(k.clone(), v.clone());
        }
        Self { rendered, translations }
    }

    pub(crate) fn back_annotate(&mut self) {
        self.rendered.vdom.back_annotate(&self.translations);
    }
}

#[derive(Hash, Eq, Debug)]
struct ListenerKey {
    id: Id,
    name: String,
}

impl ListenerKey {
    fn new<M: 'static + Send>(listener: &Listener<M>) -> Self {
        Self {
            id: listener.node_id,
            name: listener.event_name.clone()
        }
    }
}

impl PartialEq for ListenerKey {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.name == other.name
    }
}

pub(crate) struct RenderedState<A: App> {
    vdom: Option<VNode>,
    subscriptions: HashMap<Id, Subscription<A::Message>>,
    listeners: HashMap<ListenerKey, Listener<A::Message>>,
    components: HashMap<Id, ComponentContainer<A::Message>>,
}


impl<A: App> RenderedState<A> {
    pub(crate) fn new() -> Self {
        Self {
            vdom: None,
            subscriptions: Default::default(),
            listeners: Default::default(),
            components: Default::default(),
        }
    }

    pub(crate) fn take_vdom(&mut self) -> Option<VNode> {
        self.vdom.take()
    }

    pub(crate) fn get_listener(&self, target: &Id, name: &str) -> Option<&Listener<A::Message>>{
        let key = ListenerKey { id: target.clone(), name: name.into() };
        self.listeners.get(&key)
    }

    pub(crate) fn get_subscription(&self, event_id: &Id) -> Option<&Subscription<A::Message>> {
        self.subscriptions.get(&event_id)
    }

    pub(crate) fn apply(&mut self, mut frame: Frame<A>) {
        self.listeners.clear();
        self.subscriptions.clear();
        self.vdom = Some(frame.rendered.vdom);

        for item in frame.rendered.data.drain(..) {
            match item {
                ResultItem::Listener(listener) => {
                    let key = if let Some(new_id) = frame.translations.get(&listener.node_id) {
                        ListenerKey {
                            id: *new_id,
                            name: listener.event_name.clone(),
                        }
                    } else {
                        ListenerKey {
                            id: listener.node_id,
                            name: listener.event_name.clone(),
                        }
                    };
                    self.listeners.insert(key, listener);
                },
                ResultItem::Subscription(id, subs) => {
                    self.subscriptions.insert(id, subs);
                },
                ResultItem::Component(_) => {},
                ResultItem::VDom(_, _) => {},
            }
        }
    }
}
