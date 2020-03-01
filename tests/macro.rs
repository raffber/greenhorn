use greenhorn::html;
use greenhorn::prelude::{ElementBuilder, Node};

#[test]
fn test_opening_closing_macro() {
    let x: ElementBuilder<()> = html! ( <div> </div> );
    let x: Node<()> = x.into();
    println!("{:?}", x);
//    let x = html! ( <div> </> );
//    let x = html! ( <div /> );
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
