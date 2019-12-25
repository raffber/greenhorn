use super::*;

#[test]
fn parse_document() {
    let node = parse_from_string::<()>("<ul><li>Hello, World!</li></ul>").unwrap();
    if let Node::Element(elem) = node.first().unwrap() {
        println!("{:?}", elem.children);
        assert_eq!(elem.tag.as_ref().unwrap(), "ul");
    } else {
        panic!()
    }
}
