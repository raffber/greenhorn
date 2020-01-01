use crate::vdom::{Path, EventHandler, VElement, VNode};
use crate::node::{Node, ComponentContainer, ElementMap, ComponentMap};

use crate::{App, Id, Updated};
use crate::listener::Listener;
use crate::event::Subscription;
use std::collections::{HashMap, HashSet};
use crate::runtime::dag::Dag;

pub(crate) enum ResultItem<A: App> {
    Listener( Listener<A::Message> ),
    Subscription( Id, Subscription<A::Message> ),
    Component( ComponentContainer<A::Message> ),
    VDom( Id, Path, VNode ),
}



struct RenderedComponent<A: App> {
    component: ComponentContainer<A::Message>,
    vdom: VNode,
    listeners: Vec<Id>, /// Node ids of all DomEvent listeners
    subscriptions: Vec<Id>, /// Event-ids this component is subscribed to
    children: Vec<Id>, /// list of component ids which are direct children of this component
}

impl<A: App> RenderedComponent<A> {
    fn new(comp: ComponentContainer<A::Message>, dom: Node<A::Message>) -> (Self, Vec<ResultItem<A>>) {
        let mut result = Vec::new();
        let vdom = Self::render_recursive(dom, &mut result)
            .expect("Expected an actual DOM to render.");

        let mut subs = Vec::with_capacity(result.len());
        let mut listeners = Vec::with_capacity(result.len());
        let mut children = Vec::with_capacity(result.len());

        for item in &result {
            match item {
                ResultItem::Listener(listener) => {
                    listeners.push(listener.node_id)
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

pub(crate) struct RenderResult<A: App> {
    pub(crate) listeners: HashMap<ListenerKey, Listener<A::Message>>,
    pub(crate) subscriptions: HashMap<Id, Subscription<A::Message>>,
    pub(crate) components: HashMap<Id, RenderedComponent<A::Message>>,
    pub(crate) root: VNode,
    pub(crate) dag: Dag,
}

impl<A: App> RenderResult<A> {
    pub(crate) fn from_root(root_rendered: Node<A::Message>, changes: Updated) -> Self {
        todo!()
    }

    pub(crate) fn from_frame(last: Frame<A>, changes: Updated) -> Self {
        let invalidated = changes.into();
        let ancestor = last.rendered.dag.find_common_ancestor(&invalidated);
        if ancestor == last.rendered.dag.root() {
            // special case of root re-render
            todo!();
            // return ...
        }

        let children = last.rendered.dag.get_children(&changes.into());
        let new_dag = last.rendered.dag.clone();
        new_dag.remove_all_children(ancestor);

        // fetch the component to be rendered and render into the DOM
        let component = last.rendered.components.get(&ancestor).unwrap().clone();
        let dom = component.render();
        let rendered = RenderedComponent::new(component, dom);

        let mut ret = Self {
            listeners: Default::default(),
            subscriptions: Default::default(),
            components: Default::default(),
            root: last.rendered.root,
            dag: new_dag,
        };

        // ... then render it into a vdom
        let mut result = Vec::new();
        let ancestor_rendered = ret.render_recursive(dom, &mut result)
            .expect("Expect non-empty DOM from component");

        let new_components: HashSet<Id> = last.rendered.components.iter()
            .map(|x| *x.0)
            .filter(|x| children.contains(x))
            .collect();


    }

    fn update(&mut self, data: Vec<ResultItem<A>>) {
        todo!()
    }

    fn submit(&mut self, result: ResultItem<A>) {
        self.data.push(result);
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
