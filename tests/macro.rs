use greenhorn::html;

#[test]
fn test_macro() {
    let x = html! ( 1 + 1 );
    assert_eq!(x, 2);
}