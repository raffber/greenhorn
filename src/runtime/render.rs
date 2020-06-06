//! This module implements facilities to render components to a VDOM.
//!
//! Specifically, it converts [Node<T>](../../node/struct.Node.html) to a VDOM, which is
//! a recursive data structure of [VNode](../enum.VNode.html) objects, and a list of
//! non-DOM items as represented by the [ResultItem](enum.ResultItem.html) enum.
//! Two "modes" are available:
//!  * The VDOM is rendered from scratch without a previous render
//!  * The VDOM is rendered based on a previous render and only a limited set of marked components
//!     are re-rendered. The remaining components are transferred to the newly created VDOM.
//!

use crate::blob::Blob;
use crate::component::{ComponentContainer, ComponentMap};
use crate::element::ElementMap;
use crate::event::Subscription;
use crate::listener::{Listener, ListenerKey, Rpc};
use crate::node::{Node, NodeItems};
use crate::runtime::component::RenderedComponent;
use crate::runtime::metrics::Metrics;
use crate::runtime::state::Frame;
use crate::vdom::{EventHandler, Path, VElement, VNode};
use crate::{App, Id};
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::sync::Arc;

// TODO: currently an event cannot be subscribed to multiple times
// since we store the event_id as the key to find a single subscription
// however, we should use a subscription list as value

/// Renders a component from scratch and emit a set of `ResultItem`s.
pub(crate) fn render_component<A: App>(
    dom: Node<A::Message>,
    result: &mut Vec<ResultItem<A>>,
) -> Vec<VNode> {
    let mut path = Path::new();
    render_recursive(dom, result, &mut path)
}

/// Recursively renders an arbitrary node.
/// Non-DOM elements will be pushed into `result`.
fn render_recursive<A: App>(
    dom: Node<A::Message>,
    result: &mut Vec<ResultItem<A>>,
    path: &mut Path,
) -> Vec<VNode> {
    match dom.0 {
        NodeItems::ElementMap(mut elem) => {
            if let Some(x) = render_element(&mut *elem.inner, result, path) {
                vec![x]
            } else {
                vec![]
            }
        }
        NodeItems::Component(comp) => {
            let id = comp.id();
            result.push(ResultItem::Component(comp, path.clone()));
            vec![VNode::Placeholder(id, path.clone())]
        }
        NodeItems::Text(text) => vec![VNode::text(text)],
        NodeItems::Element(mut elem) => {
            if let Some(x) = render_element(&mut elem, result, path) {
                vec![x]
            } else {
                vec![]
            }
        }
        NodeItems::EventSubscription(event_id, subs) => {
            result.push(ResultItem::Subscription(event_id, subs));
            Vec::new()
        }
        NodeItems::Blob(blob) => {
            result.push(ResultItem::Blob(blob));
            Vec::new()
        }
        NodeItems::FlatMap(mut nodes) => nodes
            .drain(..)
            .flat_map(|x| render_recursive(x, result, path))
            .collect(),
    }
}

/// Recursively renders an element into a VNode.
/// Non-tree elements will be pushed into `result`.
fn render_element<A: App>(
    elem: &mut dyn ElementMap<A::Message>,
    result: &mut Vec<ResultItem<A>>,
    path: &mut Path,
) -> Option<VNode> {
    let mut children = Vec::new();
    path.push(0);
    for (k, child) in elem.take_children().drain(..).enumerate() {
        path.pop();
        path.push(k);
        let child = render_component(child, result);
        children.extend(child);
    }
    let mut events = Vec::new();
    for listener in elem.take_listeners().drain(..) {
        events.push(EventHandler::from_listener(&listener));
        result.push(ResultItem::Listener(listener));
    }
    if let Some(rpc) = elem.take_rpc() {
        result.push(ResultItem::Rpc(rpc));
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

/// Non-DOM items which are emitted from rendering a `Node<T>` data structure.
pub(crate) enum ResultItem<A: App> {
    Listener(Listener<A::Message>),
    Subscription(Id, Subscription<A::Message>),
    Component(ComponentContainer<A::Message>, Path),
    Blob(Blob),
    Rpc(Rpc<A::Message>),
}

/// Collects the result of a render operation.
///
/// This is a collection of Non-DOM items, such as DOM-event listeners or event subscriptions,
/// a root-VDOM and a list of rendered components.
pub(crate) struct RenderResult<A: App> {
    pub(crate) listeners: HashMap<ListenerKey, Listener<A::Message>>,
    pub(crate) subscriptions: HashMap<Id, Subscription<A::Message>>,
    pub(crate) blobs: HashMap<Id, Blob>,
    pub(crate) rpcs: HashMap<Id, Rpc<A::Message>>,
    components: HashMap<Id, Arc<RenderedComponent<A>>>,
    pub(crate) root_components: Vec<(Id, Path)>,
    pub(crate) root_subscriptions: Vec<Id>,
    pub(crate) root_listeners: Vec<ListenerKey>,
    pub(crate) root_blobs: Vec<Id>,
    pub(crate) root_rpcs: Vec<Id>,
    pub(crate) vdom: Arc<VNode>,
    pub(crate) rendered: HashSet<Id>,
}

impl<A: App> RenderResult<A> {
    #[cfg(test)]
    pub(crate) fn new_from_vnode(root: VNode) -> Self {
        Self {
            listeners: Default::default(),
            subscriptions: Default::default(),
            blobs: Default::default(),
            rpcs: Default::default(),
            components: Default::default(),
            root_components: Default::default(),
            root_subscriptions: vec![],
            root_listeners: vec![],
            root_blobs: vec![],
            root_rpcs: vec![],
            vdom: Arc::new(root),
            rendered: Default::default()
        }
    }

    #[cfg(test)]
    pub(crate) fn new_empty() -> Self {
        Self {
            listeners: Default::default(),
            subscriptions: Default::default(),
            blobs: Default::default(),
            rpcs: Default::default(),
            components: Default::default(),
            root_components: vec![],
            root_subscriptions: vec![],
            root_listeners: vec![],
            root_blobs: vec![],
            root_rpcs: vec![],
            vdom: Arc::new(VNode::Text("".to_string())),
            rendered: Default::default()
        }
    }

    /// Create a new RenderResult if the root component was re-rendered.
    /// Re-renders the whole component tree.
    pub(crate) fn new_from_root(root_rendered: Node<A::Message>, changes: &HashSet<Id>, metrics: &mut Metrics) -> Self {
        let mut result = Vec::new();
        let mut vdom = render_component::<A>(root_rendered, &mut result);
        if vdom.len() != 1 {
            panic!("The DOM of the root app must be represented by exactly one DOM node");
        }
        let vdom = vdom.drain(0..1).next().unwrap();

        let mut ret = Self {
            listeners: Default::default(),
            subscriptions: Default::default(),
            blobs: Default::default(),
            rpcs: Default::default(),
            components: HashMap::default(),
            root_components: Default::default(),
            root_subscriptions: vec![],
            root_listeners: vec![],
            root_blobs: vec![],
            root_rpcs: vec![],
            vdom: Arc::new(vdom),
            rendered: Default::default()
        };

        for item in result.drain(..) {
            match item {
                ResultItem::Listener(listener) => {
                    let lk = ListenerKey::new(&listener);
                    ret.listeners.insert(lk.clone(), listener);
                    ret.root_listeners.push(lk);
                }
                ResultItem::Subscription(id, subscription) => {
                    ret.subscriptions.insert(id, subscription);
                    ret.root_subscriptions.push(id);
                }
                ResultItem::Component(comp, path) => {
                    ret.root_components.push((comp.id(), path));
                    ret.render_component(None, comp, changes, metrics);
                }
                ResultItem::Blob(blob) => {
                    ret.root_blobs.push(blob.id());
                    ret.blobs.insert(blob.id(), blob);
                }
                ResultItem::Rpc(rpc) => {
                    ret.root_rpcs.push(rpc.node_id);
                    ret.rpcs.insert(rpc.node_id, rpc);
                }
            }
        }
        ret
    }

    /// Create a new RenderResult based on an old frame and a set of changed components
    /// that require re-rendering.
    ///
    /// **Precondition**: The root component must still be valid and not require a re-render
    pub(crate) fn new_from_frame(
        old: &Frame<A>,
        changes: &HashSet<Id>,
        metrics: &mut Metrics,
    ) -> Self {
        let old = &old.rendered;

        // copy all old state from root to new state
        let mut new_subs = HashMap::with_capacity(old.subscriptions.len());
        for subs in &old.root_subscriptions {
            new_subs.insert(*subs, old.subscriptions.get(&subs).unwrap().clone());
        }

        let mut new_listeners = HashMap::with_capacity(old.listeners.len());
        for listener in &old.root_listeners {
            new_listeners.insert(listener.clone(), old.listeners.get(&listener).unwrap().clone());
        }

        let mut new_rpcs = HashMap::with_capacity(old.rpcs.len());
        for rpc in &old.root_rpcs {
            new_rpcs.insert(*rpc, old.rpcs.get(&rpc).unwrap().clone());
        }

        let mut new_blobs = HashMap::with_capacity(old.blobs.len());
        for blob in &old.root_blobs {
            new_blobs.insert(*blob, old.blobs.get(&blob).unwrap().clone());
        }

        let mut ret = Self {
            listeners: new_listeners,
            subscriptions: new_subs,
            blobs: new_blobs,
            rpcs: new_rpcs,
            components: HashMap::with_capacity(old.components.len() * 2),
            root_components: old.root_components.clone(),
            root_subscriptions: old.root_subscriptions.clone(),
            root_listeners: old.root_listeners.clone(),
            root_blobs: old.root_blobs.clone(),
            root_rpcs: old.root_rpcs.clone(),
            vdom: old.vdom.clone(),
            rendered: Default::default()
        };

        // iterate over all components and check / render them recursively
        for (id, _) in &old.root_components {
            let comp = old.components.get(id).unwrap();
            ret.render_component_from_old(Some(old), comp.component(), changes, metrics);
        }

        ret
    }

    /// Renders a component and registers its results into the current object.
    fn render_component(
        &mut self,
        old: Option<&RenderResult<A>>,
        comp: ComponentContainer<A::Message>,
        changes: &HashSet<Id>,
        metrics: &mut Metrics,
    ) {
        println!("Render changed!!!");
        let id = comp.id();
        self.rendered.insert(id);
        let (rendered, mut result) = RenderedComponent::new(comp, metrics);
        self.components.insert(id, Arc::new(rendered));

        for item in result.drain(..) {
            match item {
                ResultItem::Listener(listener) => {
                    self.listeners.insert(ListenerKey::new(&listener), listener);
                }
                ResultItem::Subscription(id, subscription) => {
                    self.subscriptions.insert(id, subscription);
                }
                ResultItem::Component(comp, _) => {
                    self.render_component_from_old(old, comp, changes, metrics);
                }
                ResultItem::Blob(blob) => {
                    self.blobs.insert(blob.id(), blob);
                }
                ResultItem::Rpc(rpc) => {
                    self.rpcs.insert(rpc.node_id, rpc);
                }
            }
        }
    }

    /// Renders a component which was not changed, thus re-using all of its VDom and state.
    fn render_unchanged_component(
        &mut self,
        old: &RenderResult<A>,
        comp: ComponentContainer<A::Message>,
        changes: &HashSet<Id>,
        metrics: &mut Metrics,
    ) {
        // transfer all child items of this component to the new result.
        let id = comp.id();
        let old_render = old.components.get(&id).unwrap();
        for (child, _) in old_render.children() {
            let old_comp = old.components.get(child).unwrap();
            self.render_component_from_old(Some(old), old_comp.component(), changes, metrics)
        }
        for key in old_render.listeners() {
            let listener = old.listeners.get(key).unwrap();
            self.listeners.insert(key.clone(), listener.clone());
        }
        for rpc_id in old_render.rpcs() {
            let rpc = old.rpcs.get(rpc_id).unwrap();
            self.rpcs.insert(*rpc_id, rpc.clone());
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
    }

    /// Renders a component based on an old RenderResult
    fn render_component_from_old(
        &mut self,
        old: Option<&RenderResult<A>>,
        comp: ComponentContainer<A::Message>,
        changes: &HashSet<Id>,
        metrics: &mut Metrics,
    ) {
        let id = comp.id();
        if let Some(old) = old {
            if !changes.contains(&id) && old.components.contains_key(&id) {
                self.render_unchanged_component(old, comp, changes, metrics);
            } else {
                self.render_component(Some(old), comp, changes, metrics);
            }
        } else {
            self.render_component(old, comp, changes, metrics);
        }
    }

    pub(crate) fn get_component_vdom(&self, component_id: Id) -> Option<&VNode> {
        self.components.get(&component_id).map(|x| x.vdom())
    }

    pub(crate) fn get_rendered_component(&self, component_id: Id) -> Option<&RenderedComponent<A>> {
        self.components.get(&component_id).map(|x| x.deref())
    }
}
