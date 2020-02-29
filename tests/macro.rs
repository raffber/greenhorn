use greenhorn::html;

#[test]
fn test_opening_closing_macro() {
    let x = html! ( <div> </div> );
    assert_eq!(x, 2);
    let x = html! ( <div> </> );
    assert_eq!(x, 2);
    let x = html! ( <div /> );
    assert_eq!(x, 2);
}

#[test]
fn test_attr() {
    let x = html! ( <div foo=bar> </div> );
    assert_eq!(x, 2);
    let x = html! ( <div foo="bar"> </div> );
    assert_eq!(x, 2);
    let x = html! ( <div foo=123> </div> );
    assert_eq!(x, 2);
}


#[test]
fn test_expr_attr() {
    let x = html! ( <div foo={"foo"}> </div> );
    assert_eq!(x, 2);
}
