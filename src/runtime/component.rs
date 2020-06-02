use crate::component::{ComponentContainer, ComponentMap};
use crate::listener::ListenerKey;
use crate::runtime::metrics::Metrics;
use crate::runtime::render::{render_component, ResultItem};
use crate::vdom::{Path, VNode};
use crate::{App, Id};

/// Captures the rendered state of a component.
///
/// This object keeps a shared reference to the component.
/// At the same time it owns the VDom of the component and
/// maintains a list of all non-DOM elements created by the
/// components render operation.
pub(crate) struct RenderedComponent<A: App> {
    component: ComponentContainer<A::Message>,
    vdom: VNode,
    listeners: Vec<ListenerKey>,
    subscriptions: Vec<Id>,
    children: Vec<(Id, Path)>,
    blobs: Vec<Id>,
    rpcs: Vec<Id>,
}

impl<A: App> RenderedComponent<A> {
    pub(crate) fn new(
        comp: ComponentContainer<A::Message>,
        metrics: &mut Metrics,
    ) -> (Self, Vec<ResultItem<A>>) {
        let dom = metrics.run_comp(comp.id(), || comp.render());
        let mut result = Vec::new();
        let mut vdom = render_component(dom, &mut result);
        if vdom.len() != 1 {
            panic!("The DOM of a component must be represented by exactly one DOM node");
        }
        let vdom = vdom.drain(0..1).next().unwrap();

        let mut subs = Vec::with_capacity(result.len());
        let mut listeners = Vec::with_capacity(result.len());
        let mut children = Vec::with_capacity(result.len());
        let mut blobs = Vec::with_capacity(result.len());
        let mut rpcs = Vec::with_capacity(result.len());

        for item in &result {
            match item {
                ResultItem::Listener(listener) => {
                    let key = ListenerKey::new(listener);
                    listeners.push(key)
                }
                ResultItem::Subscription(id, _) => {
                    subs.push(*id);
                }
                ResultItem::Component(comp, path) => children.push((comp.id(), path.clone())),
                ResultItem::Blob(blob) => {
                    blobs.push(blob.id());
                }
                ResultItem::Rpc(rpc) => rpcs.push(rpc.node_id),
            }
        }

        (
            Self {
                component: comp,
                vdom,
                listeners,
                subscriptions: subs,
                children,
                blobs,
                rpcs,
            },
            result,
        )
    }

    pub(crate) fn children(&self) -> &Vec<(Id, Path)> {
        &self.children
    }

    pub(crate) fn listeners(&self) -> &Vec<ListenerKey> {
        &self.listeners
    }

    pub(crate) fn rpcs(&self) -> &Vec<Id> {
        &self.rpcs
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
