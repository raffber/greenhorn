use crate::Id;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

mod serialize;
use crate::component::Attr;
pub use serialize::serialize as patch_serialize;
use std::hash::{Hash, Hasher};
use crate::vdom::PatchItem::ChangeNamespace;


// TODO: use lifetimes instead of RC

#[derive(Clone, Eq)]
pub struct EventHandler {
    pub name: String,
    pub no_propagate: bool,
    pub prevent_default: bool,
}

impl PartialEq for EventHandler {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.no_propagate == other.no_propagate && self.prevent_default == other.prevent_default
    }
}

impl Hash for EventHandler {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[derive(Clone)]
pub struct VElement {
    pub(crate) id: Id,
    pub(crate) tag: String,
    pub(crate) attr: Vec<Attr>,
    pub(crate) events: Vec<EventHandler>,
    pub(crate) children: Vec<VNode>,
    pub(crate) namespace: Option<String>,
}

pub struct VText {
    pub id: Id,
    pub data: String,
}

#[derive(Clone)]
pub enum VNode {
    Element(Rc<VElement>),
    Text(Rc<VText>),
}

impl VNode {
    pub fn text<T: Into<String>>(data: T, id: Id) -> VNode {
        VNode::Text(Rc::new(VText {
            id,
            data: data.into(),
        }))
    }

    pub fn element(elem: VElement) -> VNode {
        VNode::Element(Rc::new(elem))
    }
}

pub enum PatchItem {
    AppendChild(VNode),
    Replace(VNode),
    ChangeText(Rc<VText>),
    TranslateId(Id),
    Descend(),
    RemoveChildren(),
    TruncateChildren(),
    NextNode(),
    RemoveAttribute(String),
    AddAtrribute(String, String),
    ReplaceAttribute(String, String),
    RemoveEvent(EventHandler),
    AddEvent(EventHandler),
    ChangeNamespace(Option<String>),
}

pub struct Patch {
    pub items: Vec<PatchItem>,
    pub translations: Vec<(Id, Id)>,
}

impl Patch {
    fn new() -> Self {
        Patch {
            items: vec![],
            translations: vec![],
        }
    }

    pub fn from_dom(vnode: VNode) -> Self {
        let mut patch = Patch::new();
        patch.push(PatchItem::Replace(vnode));
        patch
    }

    fn push(&mut self, item: PatchItem) {
        self.items.push(item)
    }

    fn translate(&mut self, from: Id, to: Id) {
        self.translations.push((from, to));
    }
}

impl VNode {
    pub fn from_string<T: Into<String>>(id: Id, s: T) -> VNode {
        let text = Rc::new(VText {
            id: id.clone(),
            data: s.into(),
        });
        VNode::Text(text)
    }

    fn id(&self) -> Id {
        match self {
            VNode::Element(e) => e.id.clone(),
            VNode::Text(e) => e.id.clone(),
        }
    }
}

pub fn diff(old: VNode, new: VNode) -> Patch {
    let mut patch = Patch::new();
    diff_recursive(old, new, &mut patch);
    patch
}

fn diff_attrs(old: Rc<VElement>, new: Rc<VElement>, patch: &mut Patch) {
    let mut old_kv = HashMap::new();
    for attr in old.attr.iter() {
        old_kv.insert(&attr.key, &attr.value);
    }
    let mut new_kv = HashMap::new();
    for attr in new.attr.iter() {
        new_kv.insert(&attr.key, &attr.value);
    }

    for attr in new.attr.iter() {
        if let Some(&old_v) = old_kv.get(&attr.key) {
            if old_v != &attr.value {
                let p = PatchItem::ReplaceAttribute(attr.key.clone(), attr.value.clone());
                patch.push(p);
            }
        } else {
            let p = PatchItem::AddAtrribute(attr.key.clone(), attr.value.clone());
            patch.push(p);
        }
    }

    for attr in old.attr.iter() {
        if !new_kv.contains_key(&attr.key) {
            let p = PatchItem::RemoveAttribute(attr.key.clone());
            patch.push(p);
        }
    }
}

fn diff_children(old: Rc<VElement>, new: Rc<VElement>, patch: &mut Patch) {
    if old.children.is_empty() && new.children.is_empty() {
        return;
    }

    if !old.children.is_empty() && new.children.is_empty() {
        patch.push(PatchItem::RemoveChildren());
        return;
    }

    patch.push(PatchItem::Descend());

    // diff common items
    let n_new = new.children.len();
    let n_old = old.children.len();
    let common_len = n_old.min(n_new);
    let range = 0..common_len;
    for k in range {
        let old_node = old.children.get(k).unwrap();
        let new_node = new.children.get(k).unwrap();
        diff_recursive(old_node.clone(), new_node.clone(), patch);
        patch.push(PatchItem::NextNode());
    }

    if n_old > n_new {
        patch.push(PatchItem::TruncateChildren());
        
    } else if n_new > n_old {
        let range = (n_new - n_old)..n_new;
        for k in range {
            let new_node = new.children.get(k).unwrap();
            patch.push(PatchItem::AppendChild(new_node.clone()))
        }
    }
}

fn diff_events(old: Rc<VElement>, new: Rc<VElement>, patch: &mut Patch) {
    let mut old_evts = HashSet::new();
    for evt in &old.events {
        old_evts.insert(evt);
    }
    let mut new_evts = HashSet::new();
    for evt in &new.events {
        new_evts.insert(evt);
    }
    for evt in old_evts.iter() {
        let evt = *evt;
        if !new_evts.contains(evt) {
            patch.push(PatchItem::RemoveEvent(evt.clone()));
        }
    }
    for evt in new_evts {
        if !old_evts.contains(evt) {
            patch.push(PatchItem::AddEvent(evt.clone()));
        }
    }
}

fn diff_recursive(old: VNode, new: VNode, patch: &mut Patch) {
    match (old, new) {
        (VNode::Element(elem_old), VNode::Element(elem_new)) => {
            if elem_old.tag != elem_new.tag {
                patch.push(PatchItem::Replace(VNode::Element(elem_new)))
            } else {
                diff_attrs(elem_old.clone(), elem_new.clone(), patch);
                diff_events(elem_old.clone(), elem_new.clone(), patch);
                let _new_id = (*elem_new).id.clone();
                diff_children(elem_old.clone(), elem_new.clone(), patch);
                if elem_old.namespace != elem_new.namespace {
                    patch.push(ChangeNamespace(elem_new.namespace.clone()))
                }
                patch.translate(elem_new.id, elem_old.id);
            }
        }
        (VNode::Text(_), VNode::Text(elem_new)) => {
            patch.push(PatchItem::ChangeText(elem_new))
        }
        (_, new) => patch.push(PatchItem::Replace(new)),
    }
}
