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

#[test]
fn test_attr() {
//    let x = html! ( <div foo=bar> </div> );
//    let x = html! ( <div foo="bar"> </div> );
//    let x = html! ( <div foo=123> </div> );
}


#[test]
fn test_expr_attr() {
//    let x = html! ( <div foo={"foo"}> </div> );
}


#[test]
fn test_class_attr() {
//    let x = html! ( <div .foo> </div> );
}


#[test]
fn test_id_attr() {
//    let x = html! ( <div #foo> </div> );
}


#[test]
fn test_listener_attr() {
//    let x = html! ( <div @foo> </div> );
}

#[test]
fn test_dashed_name() {
//    let x = html! ( <foo-bar /> );
}
