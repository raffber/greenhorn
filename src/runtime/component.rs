use crate::{App, Id};
use crate::vdom::{VNode, EventHandler, VElement};
use crate::runtime::render::{ResultItem, ListenerKey};
use crate::node::{Node, ComponentContainer, ElementMap, ComponentMap};

pub(crate) struct RenderedComponent<A: App> {
    component: ComponentContainer<A::Message>,
    vdom: VNode,
    listeners: Vec<ListenerKey>,
    subscriptions: Vec<Id>,
    children: Vec<Id>,
    blobs: Vec<Id>,
}

impl<A: App> RenderedComponent<A> {
    pub(crate) fn new(comp: ComponentContainer<A::Message>) -> (Self, Vec<ResultItem<A>>) {
        let dom = comp.render();
        let mut result = Vec::new();
        let vdom = Self::render_recursive(dom, &mut result)
            .expect("Expected an actual DOM to render.");

        let mut subs = Vec::with_capacity(result.len());
        let mut listeners = Vec::with_capacity(result.len());
        let mut children = Vec::with_capacity(result.len());
        let mut blobs = Vec::with_capacity(result.len());

        for item in &result {
            match item {
                ResultItem::Listener(listener) => {
                    let key = ListenerKey::new(listener);
                    listeners.push(key)
                },
                ResultItem::Subscription(id, _) => {
                    subs.push(id.clone());
                },
                ResultItem::Component(comp) => {
                    children.push(comp.id())
                },
                ResultItem::Blob(blob) => {
                    blobs.push(blob.id());
                }
            }
        }

        (Self {
            component: comp, vdom, listeners,
            subscriptions: subs, children, blobs,
        }, result)
    }

    pub(crate) fn id(&self) -> Id {
        self.component.id()
    }

    pub(crate) fn children(&self) -> &Vec<Id> {
        &self.children
    }

    pub(crate) fn listeners(&self) -> &Vec<ListenerKey> {
        &self.listeners
    }

    pub(crate) fn subscriptions(&self) -> &Vec<Id> {
        &self.subscriptions
    }

    pub(crate) fn vdom(&self) -> &VNode {
        &self.vdom
    }

    pub(crate) fn blobs(&self) -> &Vec<Id> {
        &self.blobs
    }

    pub(crate) fn component(&self) -> ComponentContainer<A::Message> {
        self.component.clone()
    }


    pub(crate) fn render_recursive(dom: Node<A::Message>, result: &mut Vec<ResultItem<A>>) -> Option<VNode> {
        match dom {
            Node::ElementMap(mut elem) => Self::render_element(&mut *elem, result),
            Node::Component(comp) => {
                let id = comp.id();
                result.push( ResultItem::Component(comp) );
                Some(VNode::Placeholder(id))
            }
            Node::Text(text) => Some(VNode::text(text)),
            Node::Element(mut elem) => Self::render_element(&mut elem, result),
            Node::EventSubscription(event_id, subs) => {
                result.push( ResultItem::Subscription(event_id, subs) );
                None
            }
            Node::Blob(blob) => {
                result.push( ResultItem::Blob(blob) );
                None
            }
        }
    }

    fn render_element(elem: &mut dyn ElementMap<A::Message>, result: &mut Vec<ResultItem<A>>) -> Option<VNode> {
        let mut children = Vec::new();
        for (_, child) in elem.take_children().drain(..).enumerate() {
            let child = Self::render_recursive(child, result);
            if let Some(child) = child {
                children.push(child);
            }
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
            js_events: elem.take_js_events(),
            events,
            children,
            namespace: elem.take_namespace(),
        }))
    }
}
