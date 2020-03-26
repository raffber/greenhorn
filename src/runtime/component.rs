use crate::{App, Id};
use crate::vdom::VNode;
use crate::runtime::render::{ResultItem, ListenerKey, render_recursive};
use crate::node::{ComponentContainer, ComponentMap};

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
        let vdom = render_recursive(dom, &mut result)
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
}
