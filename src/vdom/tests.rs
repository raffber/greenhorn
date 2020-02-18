use super::*;

use assert_matches::assert_matches;
use std::fs;
use crate::runtime::{Frame, RenderResult};
use std::collections::HashSet;
use crate::{App, Updated, Render};
use crate::mailbox::Mailbox;
use crate::node::Node;

struct DummyApp;

impl App for DummyApp {
    fn update(&mut self, _msg: Self::Message, _mailbox: Mailbox<Self::Message>) -> Updated {
        unimplemented!()
    }
}

impl Render for DummyApp {
    type Message = ();

    fn render(&self) -> Node<Self::Message> {
        unimplemented!()
    }
}

fn diff<'a, A: App>(old: &'a Frame<A>, new: &'a RenderResult<A>) -> Patch<'a> {
    let patch = Differ::new(old, new, HashSet::new());
    patch.diff()
}

#[test]
fn test_remove_attr() {
    let id_a = Id::new();
    let id_b = Id::new();
    let elem_a = VNode::element(VElement {
        id: id_a,
        tag: "foo".into(),
        attr: vec![Attr::new("foo", "bar")],
        js_events: vec![],
        events: vec![],
        children: vec![],
        namespace: None,
    });

    let elem_b = VNode::element(VElement {
        id: id_b,
        tag: "foo".into(),
        attr: vec![],
        js_events: vec![],
        events: vec![],
        children: vec![],
        namespace: None,
    });
    let old = Frame::<DummyApp>::from_vnode(elem_a);
    let new = RenderResult::<DummyApp>::from_vnode(elem_b);
    let patch = diff(&old, &new);

    assert_eq!(patch.translations.len(), 1);
    assert_eq!(patch.translations.get(&id_b).unwrap(), &id_a);
    assert_eq!(patch.items.len(), 1);
    if let PatchItem::RemoveAttribute(x) = &patch.items[0] {
        assert_eq!(&"foo".to_string(), x);
    } else {
        panic!();
    }
}

#[test]
fn test_add_attr() {
    let id_a = Id::new();
    let id_b = Id::new();
    let elem_a = VNode::element(VElement {
        id: id_a,
        tag: "foo".into(),
        attr: vec![],
        js_events: vec![],
        events: vec![],
        children: vec![],
        namespace: None,
    });

    let elem_b = VNode::element(VElement {
        id: id_b,
        tag: "foo".into(),
        attr: vec![Attr::new("foo", "bar")],
        js_events: vec![],
        events: vec![],
        children: vec![],
        namespace: None,
    });

    let old = Frame::<DummyApp>::from_vnode(elem_a);
    let new = RenderResult::<DummyApp>::from_vnode(elem_b);
    let patch = diff(&old, &new);

    assert_eq!(patch.translations.len(), 1);
    assert_eq!(patch.translations.get(&id_b).unwrap(), &id_a);
    assert_eq!(patch.items.len(), 1);
    if let PatchItem::AddAtrribute(key, value) = &patch.items[0] {
        assert_eq!(&"foo".to_string(), key);
        assert_eq!(&"bar".to_string(), value);
    } else {
        panic!();
    }
}

#[test]
fn test_change_attr() {
    let id_a = Id::new();
    let id_b = Id::new();
    let elem_a = VNode::element(VElement {
        id: id_a,
        tag: "foo".into(),
        attr: vec![Attr::new("foo", "bla")],
        js_events: vec![],
        events: vec![],
        children: vec![],
        namespace: None,
    });

    let elem_b = VNode::element(VElement {
        id: id_b,
        tag: "foo".into(),
        attr: vec![Attr::new("foo", "bar")],
        js_events: vec![],
        events: vec![],
        children: vec![],
        namespace: None,
    });

    let old = Frame::<DummyApp>::from_vnode(elem_a);
    let new = RenderResult::<DummyApp>::from_vnode(elem_b);
    let patch = diff(&old, &new);

    assert_eq!(patch.translations.len(), 1);
    assert_eq!(patch.translations.get(&id_b).unwrap(), &id_a);
    assert_eq!(patch.items.len(), 1);
    if let PatchItem::ReplaceAttribute(key, value) = &patch.items[0] {
        assert_eq!(&"foo".to_string(), key);
        assert_eq!(&"bar".to_string(), value);
    } else {
        panic!();
    }
}

#[test]
fn test_change_tag() {
    let elem_a = VNode::element(VElement {
        id: Id::new(),
        tag: "foo".into(),
        attr: vec![],
        js_events: vec![],
        events: vec![],
        children: vec![],
        namespace: None,
    });

    let elem_b = VNode::element(VElement {
        id: Id::new(),
        tag: "bar".into(),
        attr: vec![],
        js_events: vec![],
        events: vec![],
        children: vec![],
        namespace: None,
    });

    let old = Frame::<DummyApp>::from_vnode(elem_a);
    let new = RenderResult::<DummyApp>::from_vnode(elem_b);
    let patch = diff(&old, &new);

    assert_eq!(patch.translations.len(), 0);
    assert_eq!(patch.items.len(), 1);
    if let PatchItem::Replace(VNode::Element(node)) = &patch.items[0] {
        assert_eq!(node.id, new.root.id());
        assert_eq!(node.tag, "bar");
    } else {
        panic!()
    }
}

#[test]
fn test_add_event() {
    let elem_a = VNode::element(VElement {
        id: Id::new(),
        tag: "foo".into(),
        attr: vec![],
        js_events: vec![],
        events: vec![],
        children: vec![],
        namespace: None,
    });

    let elem_b = VNode::element(VElement {
        id: Id::new(),
        tag: "foo".into(),
        attr: vec![],
        js_events: vec![],
        events: vec![EventHandler {
            name: "click".to_string(),
            no_propagate: true,
            prevent_default: false,
        }],
        children: vec![],
        namespace: None,
    });

    let old = Frame::<DummyApp>::from_vnode(elem_a);
    let new = RenderResult::<DummyApp>::from_vnode(elem_b);
    let patch = diff(&old, &new);

    assert_eq!(patch.items.len(), 1);
    if let PatchItem::Replace(_) = patch.items[0] {
    } else {
        panic!()
    }
}

#[test]
fn test_add_child() {
    let elem_a = VNode::element(VElement {
        id: Id::new(),
        tag: "foo".into(),
        attr: vec![],
        js_events: vec![],
        events: vec![],
        children: vec![],
        namespace: None,
    });

    let elem_b = VNode::element(VElement {
        id: Id::new(),
        tag: "foo".into(),
        attr: vec![],
        js_events: vec![],
        events: vec![],
        children: vec![VNode::element(VElement {
            id: Id::new(),
            tag: "bla".into(),
            attr: vec![],
            js_events: vec![],
            events: vec![],
            children: vec![],
            namespace: None,
        })],
        namespace: None,
    });

    let old = Frame::<DummyApp>::from_vnode(elem_a);
    let new = RenderResult::<DummyApp>::from_vnode(elem_b);
    let patch = diff(&old, &new);

    assert_eq!(patch.translations.len(), 1);
    assert_eq!(patch.items.len(), 2);
    assert_matches!(&patch.items[0], PatchItem::Descend());
    if let PatchItem::AppendSibling(VNode::Element(child)) = &patch.items[1] {
        assert_eq!(child.tag, "bla");
    } else {
        panic!()
    }
}

#[test]
fn test_output_patch() {
    use crate::vdom::serialize::serialize;

    let elem_b = VNode::element(VElement {
        id: Id::new(),
        tag: "div".into(),
        attr: vec![Attr::new("class", "foo"), Attr::new("id", "bar")],
        js_events: vec![],
        events: vec![
            EventHandler {
                name: "click".to_string(),
                no_propagate: false,
                prevent_default: false,
            },
            EventHandler {
                name: "mouseenter".to_string(),
                no_propagate: false,
                prevent_default: false,
            },
        ],
        children: vec![VNode::element(VElement {
            id: Id::new(),
            tag: "span".into(),
            attr: vec![],
            js_events: vec![],
            events: vec![],
            children: vec![VNode::text("Hello, World")],
            namespace: None,
        })],
        namespace: None,
    });

    let new = RenderResult::<DummyApp>::from_vnode(elem_b);
    let patch = Patch::from_dom(&new);
    let serialized = serialize(&new, &patch);
    fs::write("test_patch.bin", serialized).expect("Unable to write file!");
}
