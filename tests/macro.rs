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