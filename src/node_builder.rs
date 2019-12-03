use crate::component::Component;
use crate::component::{Listener, Node, NodeElement, Render};
use crate::dom_event::DomEvent;
use crate::vdom::Attr;
use crate::Id;
use std::marker::PhantomData;
use std::sync::Arc;

pub struct NodeBuilder<T> {
    namespace: Option<String>,
    marker: PhantomData<T>,
}

impl<T: 'static> NodeBuilder<T> {
    pub fn new() -> NodeBuilder<T> {
        NodeBuilder {
            namespace: None,
            marker: PhantomData,
        }
    }

    pub fn new_with_ns<S: Into<String>>(namespace: S) -> NodeBuilder<T> {
        NodeBuilder {
            namespace: Some(namespace.into()),
            marker: PhantomData,
        }
    }

    pub fn elem<S>(&self, name: S) -> ElementBuilder<T>
    where
        S: Into<String>,
    {
        ElementBuilder::new(name.into(), self.namespace.clone())
    }

    pub fn text<S: Into<String>>(&self, text: S) -> TextBuilder<T> {
        TextBuilder {
            text: text.into(),
            phantom: PhantomData,
        }
    }

    pub fn mount<ChildMsg, R, Mapper>(&self, comp: &Component<R>, mapper: Mapper) -> Node<T>
    where
        ChildMsg: 'static + Send,
        R: 'static + Render<Message = ChildMsg>,
        Mapper: 'static + Fn(ChildMsg) -> T,
    {
        comp.render().map(mapper)
    }
}

impl<T: 'static> Default for NodeBuilder<T> {
    fn default() -> Self {
        NodeBuilder::new()
    }
}

pub struct TextBuilder<T> {
    text: String,
    phantom: PhantomData<T>,
}

impl<T> TextBuilder<T> {
    fn build(self) -> Node<T> {
        Node::Text(self.text)
    }
}

impl<T> From<TextBuilder<T>> for Node<T> {
    fn from(builder: TextBuilder<T>) -> Self {
        builder.build()
    }
}

pub struct ElementBuilder<T> {
    id: Id,
    tag: String,
    attrs: Vec<Attr>,
    listeners: Vec<Listener<T>>,
    children: Vec<Node<T>>,
    namespace: Option<String>,
}

impl<T: 'static> ElementBuilder<T> {
    fn new(tag: String, namespace: Option<String>) -> ElementBuilder<T> {
        ElementBuilder {
            id: Id::empty(),
            tag,
            attrs: Vec::new(),
            listeners: Vec::new(),
            children: Vec::new(),
            namespace,
        }
    }

    pub fn on<S: Into<String>, F: 'static + Fn(DomEvent) -> T>(mut self, name: S, fun: F) -> Self {
        if self.id.is_empty() {
            self.id = Id::new();
        }
        self.listeners.push(Listener {
            event_name: name.into(),
            node_id: self.id,
            fun: Arc::new(fun),
            no_propagate: false,
            prevent_default: false,
        });
        self
    }

    pub fn listener<S: Into<String>, F: 'static + Fn(DomEvent) -> T>(
        self,
        name: S,
        fun: F,
    ) -> ListenerBuilder<T> {
        ListenerBuilder {
            parent: self,
            name: name.into(),
            fun: Arc::new(fun),
            prevent_default: false,
            no_propagate: false,
        }
    }

    pub fn attr<R: Into<String>, S: Into<String>>(mut self, key: R, value: S) -> Self {
        self.attrs.push(Attr {
            key: key.into(),
            value: value.into(),
        });
        self
    }

    pub fn add<S: Into<Node<T>>>(mut self, child: S) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn add_option<S: Into<Node<T>>>(mut self, child: Option<S>) -> Self {
        if let Some(child) = child {
            self.children.push(child.into());
        }
        self
    }

    pub fn add_all<S>(mut self, children: S) -> Self
    where
        S: IntoIterator,
        S::Item: Into<Node<T>>,
     {
        for child in children {
            self.children.push(child.into());
        }
        self
    }

    pub fn text<S: Into<String>>(mut self, text: S) -> Self {
        self.children.push(Node::Text(text.into()));
        self
    }

    pub fn id<S: Into<String>>(mut self, id: S) -> Self {
        self.attrs.push(Attr {
            key: "id".into(),
            value: id.into(),
        });
        self
    }

    pub fn class<S: Into<String>>(mut self, class: S) -> Self {
        self.attrs.push(Attr {
            key: "class".into(),
            value: class.into(),
        });
        self
    }

    pub fn build(self) -> Node<T> {
        Node::Element(NodeElement {
            id: self.id,
            tag: Some(self.tag),
            attrs: Some(self.attrs),
            listeners: Some(self.listeners),
            children: Some(self.children),
            namespace: self.namespace,
        })
    }
}

pub struct ListenerBuilder<T: 'static> {
    parent: ElementBuilder<T>,
    name: String,
    fun: Arc<dyn Fn(DomEvent) -> T>,
    prevent_default: bool,
    no_propagate: bool,
}

impl<T: 'static> ListenerBuilder<T> {
    pub fn prevent_default(mut self) -> Self {
        self.prevent_default = true;
        self
    }

    pub fn no_propagate(mut self) -> Self {
        self.no_propagate = true;
        self
    }

    fn build(mut self) -> ElementBuilder<T> {
        self.parent.listeners.push(Listener {
            event_name: self.name,
            node_id: self.parent.id,
            fun: self.fun,
            no_propagate: self.no_propagate,
            prevent_default: self.prevent_default,
        });
        self.parent
    }
}

impl<T: 'static> From<ElementBuilder<T>> for Node<T> {
    fn from(builder: ElementBuilder<T>) -> Self {
        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    #[derive(Debug)]
    enum Msg {
        None,
        Clicked,
    }

    fn builder() -> NodeBuilder<Msg> {
        NodeBuilder::<Msg>::new()
    }

    #[test]
    fn test_builder() {
        let elem = builder()
            .elem("div")
            .attr("class", "foo")
            .attr("foo", "bar")
            .on("click", |_| Msg::Clicked)
            .build();

        if let Node::Element(e) = elem {
            assert_eq!(e.tag.unwrap(), "div");
            let listeners = &e.listeners.unwrap();
            let listener = listeners.get(0).unwrap();
            assert_eq!(listener.event_name, "click");
            let msg = (listener.fun)(DomEvent::Base(Id::new()));
            assert_matches!(msg, Msg::Clicked);
        } else {
            panic!()
        }
    }

    struct RenderImpl {}

    impl Render for RenderImpl {
        type Message = ();

        fn render(&self) -> Node<Self::Message> {
            self.svg().elem("svg").build()
        }
    }

    #[test]
    fn test_namespace() {
        let render = RenderImpl {};
        let node = render.render();
        if let Node::Element(e) = node {
            assert_eq!(e.namespace, Some("http://www.w3.org/2000/svg".into()))
        } else {
            panic!()
        }
    }

    #[test]
    fn test_listener() {
        let node = builder()
            .elem("div")
            .attr("test", "foo")
            .listener("click", |_| Msg::Clicked)
            .prevent_default()
            .no_propagate()
            .build()
            .attr("foo", "bar")
            .build();
        if let Node::Element(elem) = node {
            let attrs = elem.attrs.as_ref().unwrap();
            assert_eq!(attrs.len(), 2);
            assert_eq!(attrs[0].key, "test");
            assert_eq!(attrs[0].value, "foo");
            assert_eq!(attrs[1].key, "foo");
            assert_eq!(attrs[1].value, "bar");
            let listeners = elem.listeners.as_ref().unwrap();
            assert_eq!(listeners.len(), 1);
            assert_eq!(listeners[0].event_name, "click");
            assert!(listeners[0].no_propagate);
            assert!(listeners[0].prevent_default);
            assert_eq!(listeners[0].node_id, elem.id);
        } else {
            panic!()
        }
    }

    #[test]
    fn test_children() {
        let node = builder().elem("div").add(builder().elem("pre")).build();
        if let Node::Element(elem) = node {
            assert_eq!(elem.tag.unwrap(), "div");
            assert_eq!(elem.children.as_ref().unwrap().len(), 1);
            let child_node = &elem.children.as_ref().unwrap()[0];
            if let Node::Element(elem) = child_node {
                assert_eq!(elem.tag.as_ref().unwrap(), "pre");
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }
}
