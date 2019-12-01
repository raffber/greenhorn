use super::*;

use assert_matches::assert_matches;
use std::fs;

#[test]
fn test_remove_attr() {
    let id_a = Id::new();
    let id_b = Id::new();
    let elem_a = VNode::element(VElement {
        id: id_a,
        tag: "foo".into(),
        attr: vec![Attr::new("foo", "bar")],
        events: vec![],
        children: vec![],
        namespace: None
    });


    let elem_b = VNode::element(VElement {
        id: id_b,
        tag: "foo".into(),
        attr: vec![],
        events: vec![],
        children: vec![],
        namespace: None
    });

    let patch = diff(Some(&elem_a), &elem_b);
    assert_eq!(patch.translations.len(), 1);
    assert_eq!(patch.translations[0], (id_b, id_a));
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
        events: vec![],
        children: vec![],
        namespace: None
    });


    let elem_b = VNode::element(VElement {
        id: id_b,
        tag: "foo".into(),
        attr: vec![Attr::new("foo", "bar")],
        events: vec![],
        children: vec![],
        namespace: None
    });

    let patch = diff(Some(&elem_a), &elem_b);
    assert_eq!(patch.translations.len(), 1);
    assert_eq!(patch.translations[0], (id_b, id_a));
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
        events: vec![],
        children: vec![],
        namespace: None
    });


    let elem_b = VNode::element(VElement {
        id: id_b,
        tag: "foo".into(),
        attr: vec![Attr::new("foo", "bar")],
        events: vec![],
        children: vec![],
        namespace: None
    });

    let patch = diff(Some(&elem_a), &elem_b);
    assert_eq!(patch.translations.len(), 1);
    assert_eq!(patch.translations[0], (id_b, id_a));
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
        events: vec![],
        children: vec![],
        namespace: None
    });


    let elem_b = VNode::element(VElement {
        id: Id::new(),
        tag: "bar".into(),
        attr: vec![],
        events: vec![],
        children: vec![],
        namespace: None
    });

    let patch = diff(Some(&elem_a), &elem_b);
    assert_eq!(patch.translations.len(), 0);
    assert_eq!(patch.items.len(), 1);
    if let PatchItem::Replace(VNode::Element(node)) = &patch.items[0] {
        assert_eq!(node.id, elem_b.id());
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
        events: vec![],
        children: vec![],
        namespace: None
    });


    let elem_b = VNode::element(VElement {
        id: Id::new(),
        tag: "foo".into(),
        attr: vec![],
        events: vec![EventHandler {
            name: "click".to_string(),
            no_propagate: true,
            prevent_default: false
        }],
        children: vec![],
        namespace: None
    });

    let patch = diff(Some(&elem_a), &elem_b);
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
        events: vec![],
        children: vec![],
        namespace: None
    });


    let elem_b = VNode::element(VElement {
        id: Id::new(),
        tag: "foo".into(),
        attr: vec![],
        events: vec![],
        children: vec![VNode::element(VElement {
            id: Id::new(),
            tag: "bla".into(),
            attr: vec![],
            events: vec![],
            children: vec![],
            namespace: None
        })],
        namespace: None
    });

    let patch = diff(Some(&elem_a), &elem_b);
    assert_eq!(patch.translations.len(), 1);
    assert_eq!(patch.items.len(), 3);
    assert_matches!(&patch.items[0], PatchItem::Descend());
    assert_matches!(&patch.items[2], PatchItem::Ascend());
    if let PatchItem::AppendNode(VNode::Element(child)) = &patch.items[1] {
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
        events: vec![EventHandler {
            name: "click".to_string(),
            no_propagate: false,
            prevent_default: false
        },EventHandler {
            name: "mouseenter".to_string(),
            no_propagate: false,
            prevent_default: false
        }
        ],
        children: vec![VNode::element(VElement {
            id: Id::new(),
            tag: "span".into(),
            attr: vec![],
            events: vec![],
            children: vec![VNode::text("Hello, World")],
            namespace: None
        })],
        namespace: None
    });

    let patch = diff(None, &elem_b);
    let serialized = serialize(patch);
    fs::write("test_patch.bin", serialized).expect("Unable to write file!");
}