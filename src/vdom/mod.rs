
mod serialize;
mod diff;
#[cfg(test)] mod tests;

pub(crate) use diff::Differ;
use crate::{Id, App};
use std::collections::HashMap;
pub(crate) use serialize::serialize as patch_serialize;
use std::hash::{Hash, Hasher};
use crate::listener::Listener;
use crate::node::Blob;
use crate::runtime::RenderResult;


#[derive(Clone)]
pub struct Path {
    inner: Vec<usize>
}

impl Path {
    pub fn new() -> Path {
        // preallocate as length probably fairly small
        Path {
            inner: Vec::with_capacity(64)
        }
    }

    pub fn push(&mut self, idx: usize) {
        self.inner.push(idx);
    }

    pub fn pop(&mut self) -> Option<usize> {
        self.inner.pop()
    }
}

#[derive(Debug, Clone)]
pub struct Attr {
    pub key: String,
    pub value: String,
}

impl Attr {
    fn new<K: Into<String>, V: Into<String>>(key: K, value: V) -> Self {
        Attr {
            key: key.into(),
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, Eq)]
pub struct EventHandler {
    pub name: String,
    pub no_propagate: bool,
    pub prevent_default: bool,
}

impl EventHandler {
    pub(crate) fn from_listener<T>(listener: &Listener<T>) -> Self {
        EventHandler {
            name: listener.event_name.clone(),
            no_propagate: listener.no_propagate,
            prevent_default: listener.prevent_default,
        }
    }
}

impl PartialEq for EventHandler {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.no_propagate == other.no_propagate
            && self.prevent_default == other.prevent_default
    }
}

impl Hash for EventHandler {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[derive(Debug, Clone)]
pub struct VElement {
    pub(crate) id: Id,
    pub(crate) tag: String,
    pub(crate) attr: Vec<Attr>,
    pub(crate) js_events: Vec<Attr>,
    pub(crate) events: Vec<EventHandler>,
    pub(crate) children: Vec<VNode>,
    pub(crate) namespace: Option<String>,
}

#[derive(Debug, Clone)]
pub enum VNode {
    Element(VElement),
    Text(String),
    Placeholder(Id),
}

impl VNode {
    pub fn text<T: Into<String>>(data: T) -> VNode {
        VNode::Text(data.into())
    }

    pub fn element(elem: VElement) -> VNode {
        VNode::Element(elem)
    }

    pub fn replace(&mut self, path: &[usize], value: VNode) {
        match self {
            VNode::Element(elem) => {
                let idx = path[0];
                assert!(elem.children.len() <= idx as usize);
                if path.len() == 1 {
                    elem.children[idx] = value;
                } else {
                    elem.children[idx].replace(&path[1..], value);
                }
            },
            VNode::Text(_) => {panic!()},
            VNode::Placeholder(_) => {panic!()},
        }
    }

    pub fn from_string<T: Into<String>>(s: T) -> VNode {
        VNode::Text(s.into())
    }

    fn id(&self) -> Id {
        match self {
            VNode::Element(e) => e.id,
            VNode::Text(_) => Id::empty(),
            VNode::Placeholder(x) => x.clone(),
        }
    }
}

#[derive(Debug)]
pub enum PatchItem<'a> {
    AppendSibling(&'a VNode),
    Replace(&'a VNode),
    ChangeText(&'a str),
    Ascend(),
    Descend(),
    RemoveChildren(),
    TruncateSiblings(),
    NextNode(),
    RemoveAttribute(&'a str),
    AddAtrribute(&'a str, &'a str),
    ReplaceAttribute(&'a str, &'a str),
    AddBlob(Blob),
    RemoveBlob(Id),

    RemoveJsEvent(&'a str),
    AddJsEvent(&'a str, &'a str),
    ReplaceJsEvent(&'a str, &'a str),
}

impl<'a> PatchItem<'a> {
    fn is_move(&'a self) -> bool {
        match self {
            PatchItem::Ascend() => true,
            PatchItem::Descend() => true,
            PatchItem::NextNode() => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct Patch<'a> {
    pub items: Vec<PatchItem<'a>>,
    pub translations: HashMap<Id, Id>,
}

impl<'a> Patch<'a> {
    fn new() -> Self {
        Patch {
            items: vec![],
            translations: HashMap::new(),
        }
    }

    pub(crate) fn from_dom<A: App>(rendered: &'a RenderResult<A>) -> Self {
        let mut patch = Patch::new();
        patch.push(PatchItem::Replace(&rendered.root));
        for (_, v) in &rendered.blobs {
            patch.push(PatchItem::AddBlob(v.clone()));
        }
        patch
    }

    fn push(&mut self, item: PatchItem<'a>) {
        self.items.push(item)
    }

    fn translate(&mut self, from: Id, to: Id) {
        self.translations.insert(from, to);
    }

    pub fn is_empty(&self) -> bool {
        self.items.len() == 0
    }

    fn optimize(&mut self) {
        // optimize trailing moves as they are useless and trivial to optimize
        let mut cutoff = 0;
        for x in self.items.iter().rev() {
            if x.is_move() {
                cutoff += 1;
            } else {
                break;
            }
        }
        self.items.truncate(self.items.len() - cutoff);
    }
}


