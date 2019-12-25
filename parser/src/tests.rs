use super::*;

#[test]
fn parse_document() {
    let node = parse_from_string::<()>("<ul><li>Hello, World!</li></ul>").unwrap();
    if let Node::Element(elem) = node.first().unwrap() {
        assert_eq!(elem.namespace, None);
        assert_eq!(elem.tag.as_ref().unwrap(), "ul");
        assert_eq!(elem.children.as_ref().unwrap().len(), 1);
        let li_node = elem.children.as_ref().unwrap().first().unwrap();
        if let Node::Element(li_elem) = li_node {
            assert_eq!(li_elem.tag.as_ref().unwrap(), "li");
            if let Node::Text(x) = li_elem.children.as_ref().unwrap().first().unwrap() {
                assert_eq!(x, "Hello, World!");
            } else {
                panic!()
            }
        } else {
            panic!();
        }
    } else {
        panic!()
    }
}

#[test]
fn parse_svg() {
    let svg = "
    <svg height=\"100\" width=\"100\">
      <circle cx=\"50\" cy=\"50\" r=\"40\" stroke=\"black\" stroke-width=\"3\" fill=\"red\" />
    </svg>
    ";
    let node = parse_from_string::<()>(svg).unwrap();
    if let Node::Element(elem) = node.first().unwrap() {
        assert_eq!(elem.namespace.as_ref().unwrap(), "http://www.w3.org/2000/svg");
    } else {
        panic!();
    }

}
