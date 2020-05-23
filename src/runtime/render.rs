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
) -> Option<VNode> {
    let mut path = Path::new();
    render_recursive(dom, result, &mut path)
}

/// Recursively renders an arbitrary node.
/// Non-DOM elements will be pushed into `result`.
fn render_recursive<A: App>(
    dom: Node<A::Message>,
    result: &mut Vec<ResultItem<A>>,
    path: &mut Path,
) -> Option<VNode> {
    match dom.0 {
        NodeItems::ElementMap(mut elem) => render_element(&mut *elem.inner, result, path),
        NodeItems::Component(comp) => {
            let id = comp.id();
            result.push(ResultItem::Component(comp, path.clone()));
            Some(VNode::Placeholder(id, path.clone()))
        }
        NodeItems::Text(text) => Some(VNode::text(text)),
        NodeItems::Element(mut elem) => render_element(&mut elem, result, path),
        NodeItems::EventSubscription(event_id, subs) => {
            result.push(ResultItem::Subscription(event_id, subs));
            None
        }
        NodeItems::Blob(blob) => {
            result.push(ResultItem::Blob(blob));
            None
        }
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
        if let Some(child) = child {
            children.push(child);
        }
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
    Listener(Listener<A::Msg>),
    Subscription(Id, Subscription<A::Msg>),
    Component(ComponentContainer<A::Msg>, Path),
    Blob(Blob),
    Rpc(Rpc<A::Msg>),
}

/// Collects the result of a render operation.
///
/// This is a collection of Non-DOM items, such as DOM-event listeners or event subscriptions,
/// a root-VDOM and a list of rendered components.
pub(crate) struct RenderResult<A: App> {
    pub(crate) listeners: HashMap<ListenerKey, Listener<A::Msg>>,
    pub(crate) subscriptions: HashMap<Id, Subscription<A::Msg>>,
    pub(crate) blobs: HashMap<Id, Blob>,
    pub(crate) rpcs: HashMap<Id, Rpc<A::Msg>>,
    components: HashMap<Id, Arc<RenderedComponent<A>>>,
    pub(crate) root_components: Vec<(Id, Path)>,
    pub(crate) root: Arc<VNode>,
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
            root: Arc::new(root),
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
            root: Arc::new(VNode::Text("".to_string())),
        }
    }

    /// Create a new RenderResult if the root component was re-rendered.
    /// Re-renders the whole component tree.
    pub(crate) fn new_from_root(root_rendered: Node<A::Message>, metrics: &mut Metrics) -> Self {
        let mut result = Vec::new();
        let vdom =
            render_component::<A>(root_rendered, &mut result).expect("Root produced an empty DOM");

        let mut ret = Self {
            listeners: Default::default(),
            subscriptions: Default::default(),
            blobs: Default::default(),
            rpcs: Default::default(),
            components: HashMap::default(),
            root_components: Default::default(),
            root: Arc::new(vdom),
        };

        for item in result.drain(..) {
            match item {
                ResultItem::Listener(listener) => {
                    ret.listeners.insert(ListenerKey::new(&listener), listener);
                }
                ResultItem::Subscription(id, subscription) => {
                    ret.subscriptions.insert(id, subscription);
                }
                ResultItem::Component(comp, path) => {
                    ret.root_components.push((comp.id(), path));
                    ret.render_component(None, comp, None, metrics);
                }
                ResultItem::Blob(blob) => {
                    ret.blobs.insert(blob.id(), blob);
                }
                ResultItem::Rpc(rpc) => {
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
        let mut ret = Self {
            listeners: Default::default(),
            subscriptions: Default::default(),
            blobs: Default::default(),
            rpcs: Default::default(),
            components: HashMap::with_capacity(old.components.len() * 2),
            root_components: Default::default(),
            root: old.root.clone(),
        };

        let root_components = old.root_components.clone(); // XXX: workaround
        for (id, _) in &root_components {
            let comp = old.components.get(id).unwrap();
            ret.render_component_from_old(Some(old), comp.component(), Some(changes), metrics);
        }

        ret.root_components = old.root_components.clone();
        ret.root = old.root.clone();
        ret
    }

    /// Renders a component and registers its results into the current object.
    fn render_component(
        &mut self,
        old: Option<&RenderResult<A>>,
        comp: ComponentContainer<A::Message>,
        changes: Option<&HashSet<Id>>,
        metrics: &mut Metrics,
    ) {
        let id = comp.id();
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
            self.render_component_from_old(Some(old), old_comp.component(), Some(changes), metrics)
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
        changes: Option<&HashSet<Id>>,
        metrics: &mut Metrics,
    ) {
        let id = comp.id();
        if let Some(old) = old {
            let changes = changes.unwrap();
            if !changes.contains(&id) && old.components.contains_key(&id) {
                self.render_unchanged_component(old, comp, changes, metrics);
            } else {
                self.render_component(Some(old), comp, Some(changes), metrics);
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
