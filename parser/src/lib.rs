use greenhorn::prelude::{Node, ElementBuilder, NodeBuilder};
use std::future::Future;
use std::io;
use std::fs;
use scraper::Html;
use std::path::Path;
use ego_tree::NodeRef;
use scraper::Node as ScraperNode;
use std::ops::Deref;
use std::fmt::{Display, Formatter, Error};


#[derive(Debug)]
pub enum LoadError {
    Io(io::Error),
    Parse(Vec<String>),
    Empty,
}

type Result<T> = std::result::Result<T, LoadError>;

impl Display for LoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            LoadError::Io(x) => {
                write!(f, "IO error: {}", x)
            },
            LoadError::Parse(x) => {
                if x.len() == 0 {
                    write!(f, "Error during parsing step.")
                } else if x.len() == 1 {
                    write!(f, "Cannot parse: {}", x[0])
                } else {
                    write!(f, "While parsing, the following errors occured:")?;
                    for err in x {
                        write!(f, "\t{}", err)?;
                    }
                    Ok(())
                }
            },
            LoadError::Empty => {
                write!(f, "Document was emtpy")
            },
        }
    }
}

fn load_element<T: 'static + Send>(elem: &scraper::node::Element) -> ElementBuilder<T> {
    let ns: &str = elem.name.ns.as_ref();
    let builder = if ns == "http://www.w3.org/1999/xhtml" || ns == "" {
        NodeBuilder::new()
    } else {
        NodeBuilder::new_with_ns(ns)
    };
    let mut builder = builder.elem(&elem.name.local as &str);
    for (key, value) in &elem.attrs {
        builder = builder.attr(key.local.as_ref(), value);
    }
    builder.into()
}

fn load_tree<T: 'static + Send>(node: NodeRef<scraper::node::Node>) -> Option<Node<T>> {
    match node.value() {
        ScraperNode::Document => { None },
        ScraperNode::Fragment => { None },
        ScraperNode::Doctype(_) => { None },
        ScraperNode::Comment(_) => { None },
        ScraperNode::Text(txt) => { Some(Node::html().text(txt.deref())) },
        ScraperNode::Element(elem) => {
            let mut builder = load_element(elem);
            for item in node.children() {
                builder = builder.add(load_tree(item));
            }
            Some(builder.build())
        },
        ScraperNode::ProcessingInstruction(_) => { None },
    }
}

pub fn parse_from_string<T: 'static + Send>(value: &str) -> Result<Vec<Node<T>>> {
    let document = Html::parse_fragment(value.trim());
    if document.errors.len() != 0 {
        let errs = document.errors.iter().map(|x| x.to_string()).collect();
        return Err(LoadError::Parse(errs));
    }
    let value = document.tree.root().value();
    match value {
        ScraperNode::Fragment => {
            if let Some(node) = document.tree.root().children().next() {
                match node.value() {
                    ScraperNode::Text(txt) => {
                        Ok(vec![Node::html().text(txt.deref())])
                    },
                    ScraperNode::Element(_) => {
                        Ok(
                            node.children()
                            .flat_map(|x| load_tree(x))
                            .collect::<Vec<_>>()
                        )
                    },
                    _ => Err(LoadError::Empty)
                }
            } else {
                Err(LoadError::Empty)
            }
        },
        _ => panic!()
    }
}

pub fn parse_from_file_sync<T: 'static + Send>(path: &str) -> Result<Vec<Node<T>>> {
    let data = fs::read_to_string(path).map_err(LoadError::Io)?;
    parse_from_string(&data)
}

pub fn parse_from_file_async<T: 'static + Send>(path: &str) -> impl Future<Output=Result<Vec<Node<T>>>> {
    let path = path.to_string();
    async move {
        let path = Path::new(&path);
        let data = async_std::fs::read_to_string(path).await;
        let data = data.map_err(LoadError::Io)?;
        parse_from_string(&data)
    }
}

#[cfg(test)]
mod tests;
