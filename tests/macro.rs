use greenhorn::html;
use greenhorn::prelude::{ElementBuilder, Node};


fn check_node_as_div(node: ElementBuilder<()>) {
    let node = node.into();
    match node {
        Node::Element(elem) => {
            assert_eq!(elem.tag.unwrap(), "div");
            assert!(elem.namespace.is_none());
        },
        _ => panic!()
    }
}

#[test]
fn test_opening_closing_macro() {
    check_node_as_div(html! ( <div> </div> ));
    check_node_as_div(html! ( <div> </> ));
    check_node_as_div(html! ( <div /> ));
}


fn check_attr(node: Node<()>, data: &str) {
    match node {
        Node::Element(elem) => {
            assert_eq!(elem.tag.unwrap(), "div");
            let attrs = elem.attrs.as_ref().unwrap();
            assert_eq!(attrs.len(), 1);
            let attr = &attrs[0];
            assert_eq!(attr.key, "foo");
            assert_eq!(attr.value, data);
        },
        _ => panic!()
    }
}

#[test]
fn test_attr() {
    check_attr(html! ( <div foo=bar> </div> ).into(), "bar");
    check_attr(html! ( <div foo="bar"> </div> ).into(), "bar");
    check_attr(html! ( <div foo=123> </div> ).into(), "123");
}


#[test]
fn test_expr_attr() {
    let x: Node<()> = html! ( <div foo={"bar"}> </div> ).into();
    check_attr(x, "bar");

    let y = 23;
    let x: Node<()> = html! ( <div foo={100 + y}> </div> ).into();
    check_attr(x, "123");
}


#[test]
fn test_class_attr() {
    let node: Node<()> = html! ( <div .foo> </div> ).into();
    match node {
        Node::Element(elem) => {
            assert_eq!(elem.tag.unwrap(), "div");
            let attrs = elem.attrs.as_ref().unwrap();
            assert_eq!(attrs.len(), 1);
            let attr = &attrs[0];
            assert_eq!(attr.key, "class");
            assert_eq!(attr.value, "foo");
        },
        _ => panic!()
    }
}


#[test]
fn test_id_attr() {
    let node: Node<()> = html! ( <div #foo> </div> ).into();
    match node {
        Node::Element(elem) => {
            assert_eq!(elem.tag.unwrap(), "div");
            let attrs = elem.attrs.as_ref().unwrap();
            assert_eq!(attrs.len(), 1);
            let attr = &attrs[0];
            assert_eq!(attr.key, "id");
            assert_eq!(attr.value, "foo");
        },
        _ => panic!()
    }
}


#[test]
fn test_listener_attr() {
    let node: Node<()> = html! ( <div $foo="js-stuff"> </div> ).into();
    match node {
        Node::Element(elem) => {
            assert_eq!(elem.tag.unwrap(), "div");
            let js_evts = elem.js_events.as_ref().unwrap();
            assert_eq!(js_evts.len(), 1);
            let attr = &js_evts[0];
            assert_eq!(attr.key, "foo");
            assert_eq!(attr.value, "js-stuff");
        },
        _ => panic!()
    }
}

#[test]
fn test_dashed_name() {
    let node: Node<()> = html! ( <foo-bar /> ).into();
    match node {
        Node::Element(elem) => {
            assert_eq!(elem.tag.unwrap(), "foo-bar");
        },
        _ => panic!()
    }
}
