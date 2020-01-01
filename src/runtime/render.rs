use crate::vdom::{Path, EventHandler, VElement, VNode};
use crate::node::{Node, ComponentContainer, ElementMap};

use crate::{App, Id};
use crate::listener::Listener;
use crate::event::Subscription;
use std::collections::HashMap;

pub(crate) enum ResultItem<A: App> {
    Listener( Listener<A::Message> ),
    Subscription( Id, Subscription<A::Message> ),
    Component( ComponentContainer<A::Message> ),
    VDom( Path, VNode ),
}

pub(crate) struct RenderResult<A: App> {
    pub(crate) data: Vec<ResultItem<A>>,
    pub(crate) vdom: VNode,
}

impl<A: App> RenderResult<A> {
    pub(crate) fn new(dom: Node<A::Message>, path: &mut Path) -> Self {
        let mut data = Vec::new();
        let vdom = Self::render_recursive(&mut data, dom, path)
            .expect("Expected an actual DOM to render.");
        Self { data, vdom }
    }

    fn submit(&mut self, result: ResultItem<A>) {
        self.data.push(result);
    }

    fn render_recursive(data: &mut Vec<ResultItem<A>>, dom: Node<A::Message>, path: &mut Path) -> Option<VNode> {
        match dom {
            Node::ElementMap(mut elem) => RenderResult::render_element(data, &mut *elem, path),
            Node::Component(comp) => {
                let rendered = comp.render();
                let vdom = RenderResult::render_recursive(data,rendered, path)
                    .expect("Expected an actual DOM to render.");
                data.push( ResultItem::VDom(path.clone(), vdom) );
                data.push(ResultItem::Component(ComponentContainer::new(comp)));
                Some(VNode::Placeholder())
            }
            Node::Text(text) => Some(VNode::text(text)),
            Node::Element(mut elem) => RenderResult::render_element(data,&mut elem, path),
            Node::EventSubscription(event_id, subs) => {
                data.push( ResultItem::Subscription(event_id, subs) );
                None
            }
        }
    }


    fn render_element(data: &mut Vec<ResultItem<A>>, elem: &mut dyn ElementMap<A::Message>, path: &mut Path) -> Option<VNode> {
        let mut children = Vec::new();
        for (idx, child) in elem.take_children().drain(..).enumerate() {
            path.push(idx);
            let child = RenderResult::render_recursive(data, child, path);
            if let Some(child) = child {
                children.push(child);
            }
            path.pop();
        }
        let mut events = Vec::new();
        for listener in elem.take_listeners().drain(..) {
            events.push(EventHandler::from_listener(&listener));
            data.push( ResultItem::Listener(listener) );
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
