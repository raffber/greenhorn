use crate::node::Node;
use crate::vdom::Attr;
use crate::node_builder::{ElementBuilder, NodeIter};
use std::iter::{Once, once};


pub struct CheckBox<T: 'static + Send> {
    attrs: Vec<Attr>,
    classes: Vec<String>,
    checked: bool,
    html_id: Option<String>,
    node: ElementBuilder<T>,
}

impl<T: 'static + Send> CheckBox<T> {
    pub fn class<S: Into<String>>(mut self, cls: S) -> Self {
        self.classes.push(cls.into());
        self
    }

    pub fn id<S: Into<String>>(mut self, id: S) -> Self {
        self.html_id = Some(id.into());
        self
    }

    pub fn attr<R: Into<String>, S: Into<String>>(mut self, key: R, value: S) -> Self {
        let attr = Attr {
            key: key.into(),
            value: value.into()
        };
        self.attrs.push(attr);
        self
    }

    pub fn build(mut self) -> Node<T> {
        for attr in self.attrs.drain(..) {
            self.node = self.node.attr(attr.key, attr.value);
        }
        for cls in self.classes.drain(..) {
            self.node = self.node.class(cls);
        }
        if let Some(id) = self.html_id {
            self.node = self.node.id(id)
        }
        self.node.build()
    }
}

pub fn checkbox<T: 'static + Send, Fun: 'static + Send + Fn() -> T>(checked: bool, handler: Fun) -> CheckBox<T> {
    let mut node = Node::html()
        .elem("input")
        .attr("type", "checkbox")
        .attr("checked", checked.to_string());
    node = node.listener("click", move |_| handler()).prevent_default().build();
    CheckBox {
        attrs: vec![],
        classes: vec![],
        checked,
        html_id: None,
        node
    }
}

impl<T: 'static + Send> From<CheckBox<T>> for NodeIter<T, Once<Node<T>>> {
    fn from(value: CheckBox<T>) -> Self {
        NodeIter { inner: once(value.build()) }
    }
}