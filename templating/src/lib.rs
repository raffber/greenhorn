use greenhorn::prelude::{Node, ElementBuilder, NodeBuilder};
use std::future::Future;
use std::io;
use std::fs;
use scraper::Html;
use std::path::Path;
use ego_tree::NodeRef;
use scraper::Node as ScraperNode;
use std::ops::Deref;
use html5ever::{ns, namespace_url};


#[derive(Debug)]
pub enum LoadError {
    Io(io::Error),
    Parse(Vec<String>),
    Empty,
}

type Result<T> = std::result::Result<T, LoadError>;

fn load_element<T: 'static>(elem: &scraper::node::Element) -> ElementBuilder<T> {
    let ns: &str = elem.name.ns.as_ref();
    println!("{}", ns);
    let builder = NodeBuilder::new();
    let mut builder = builder.elem(&elem.name.local as &str);
    for (key, value) in &elem.attrs {
        builder = builder.attr(key.local.as_ref(), value);
    }
    builder.into()
}

fn load_tree<T: 'static>(node: NodeRef<scraper::node::Node>) -> Option<Node<T>> {
    match node.value() {
        ScraperNode::Document => { None },
        ScraperNode::Fragment => { None },
        ScraperNode::Doctype(_) => { None },
        ScraperNode::Comment(_) => { None },
        ScraperNode::Text(txt) => { Some(Node::html().text(txt.deref())) },
        ScraperNode::Element(elem) => {
            let mut builder = load_element(elem);
            for item in node.children() {
                builder = builder.add_option(load_tree(item));
            }
            Some(builder.build())
        },
        ScraperNode::ProcessingInstruction(_) => { None },
    }
}

pub fn load_from_string<T: 'static>(value: &str) -> Result<Vec<Node<T>>> {
    let document = Html::parse_fragment(value);
    if document.errors.len() != 0 {
        let errs = document.errors.iter().map(|x| x.to_string()).collect();
        return Err(LoadError::Parse(errs));
    }
    let value = document.tree.root().value();
    match value {
        ScraperNode::Fragment => {
            if let Some(node) = document.tree.root().children().next() {
                // Fragment type
                return match load_tree(node) {
                    Some(Node::Element(mut elem)) => {
                        Ok(elem.children.take().unwrap())
                    },
                    Some(Node::Text(txt)) => {Ok(vec![Node::Text(txt)])}
                    _ => Err(LoadError::Empty)
                };
            } else {
                Err(LoadError::Empty)
            }
        },
        _ => panic!()
    }
}

pub fn load_from_file_sync<T: 'static>(path: &str) -> Result<Vec<Node<T>>> {
    let data = fs::read_to_string(path).map_err(LoadError::Io)?;
    load_from_string(&data)
}

pub fn load_from_file_async<T: 'static>(path: &str) -> impl Future<Output=Result<Vec<Node<T>>>> {
    let path = path.to_string();
    async move {
        let path = Path::new(&path);
        let data = async_std::fs::read_to_string(path).await;
        let data = data.map_err(LoadError::Io)?;
        load_from_string(&data)
    }
}

#[cfg(test)]
mod tests;
