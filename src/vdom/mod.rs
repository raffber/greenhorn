use crate::Id;
use std::collections::HashMap;

mod serialize;
pub use serialize::serialize as patch_serialize;
use std::hash::{Hash, Hasher};
use crate::listener::Listener;

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
    pub(crate) events: Vec<EventHandler>,
    pub(crate) children: Vec<VNode>,
    pub(crate) namespace: Option<String>,
}

impl VElement {
    fn back_annotate(&mut self, translations: &HashMap<Id, Id>) {
        if let Some(new_id) = translations.get(&self.id) {
            self.id = *new_id;
        }
        self.children
            .iter_mut()
            .for_each(|x| x.back_annotate(translations));
    }
}

#[derive(Debug, Clone)]
pub enum VNode {
    Element(VElement),
    Text(String),
}

impl VNode {
    pub fn text<T: Into<String>>(data: T) -> VNode {
        VNode::Text(data.into())
    }

    pub fn element(elem: VElement) -> VNode {
        VNode::Element(elem)
    }

    pub fn back_annotate(&mut self, translations: &HashMap<Id, Id>) {
        match self {
            VNode::Element(elem) => elem.back_annotate(translations),
            VNode::Text(_) => {}
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
    pub translations: Vec<(Id, Id)>,
}

impl<'a> Patch<'a> {
    fn new() -> Self {
        Patch {
            items: vec![],
            translations: vec![],
        }
    }

    pub fn from_dom(vnode: &'a VNode) -> Self {
        let mut patch = Patch::new();
        patch.push(PatchItem::Replace(vnode));
        patch
    }

    fn push(&mut self, item: PatchItem<'a>) {
        self.items.push(item)
    }

    fn translate(&mut self, from: Id, to: Id) {
        self.translations.push((from, to));
    }

    pub fn is_empty(&self) -> bool {
        self.items.len() == 0
    }
}

impl VNode {
    pub fn from_string<T: Into<String>>(s: T) -> VNode {
        VNode::Text(s.into())
    }

    fn id(&self) -> Id {
        match self {
            VNode::Element(e) => e.id,
            VNode::Text(_) => Id::empty(),
        }
    }
}

pub fn diff<'a>(old: Option<&'a VNode>, new: &'a VNode) -> Patch<'a> {
    let mut patch = Patch::new();
    if let Some(old) = old {
        diff_recursive(old, new, &mut patch);
        optimize_patch(&mut patch);
    } else {
        patch.push(PatchItem::AppendSibling(new))
    }
    patch
}

fn optimize_patch(patch: &mut Patch) {
    // optimize trailing moves as they are useless and trivial to optimize
    let mut cutoff = 0;
    for x in patch.items.iter().rev() {
        if x.is_move() {
            cutoff += 1;
        } else {
            break;
        }
    }
    patch.items.truncate(patch.items.len() - cutoff);
}

fn diff_attrs<'a>(old: &'a VElement, new: &'a VElement, patch: &mut Patch<'a>) -> bool {
    let mut ret = false;

    let mut old_kv = HashMap::with_capacity(old.attr.len());
    for attr in old.attr.iter() {
        old_kv.insert(&attr.key, &attr.value);
    }
    let mut new_kv = HashMap::with_capacity(new.attr.len());
    for attr in new.attr.iter() {
        new_kv.insert(&attr.key, &attr.value);
    }

    for attr in new.attr.iter() {
        if let Some(&old_v) = old_kv.get(&attr.key) {
            if old_v != &attr.value {
                ret = true;
                let p = PatchItem::ReplaceAttribute(&attr.key, &attr.value);
                patch.push(p);
            }
        } else {
            ret = true;
            let p = PatchItem::AddAtrribute(&attr.key, &attr.value);
            patch.push(p);
        }
    }

    for attr in old.attr.iter() {
        if !new_kv.contains_key(&attr.key) {
            ret = true;
            let p = PatchItem::RemoveAttribute(&attr.key);
            patch.push(p);
        }
    }

    ret
}

#[allow(clippy::comparison_chain)]
fn diff_children<'a>(old: &'a VElement, new: &'a VElement, patch: &mut Patch<'a>) -> bool {
    if old.children.is_empty() && new.children.is_empty() {
        return false;
    }

    if !old.children.is_empty() && new.children.is_empty() {
        patch.push(PatchItem::RemoveChildren());
        return false;
    }

    let mut ret = false;
    let mut truncates = 1;
    patch.push(PatchItem::Descend());

    // diff common items
    let n_new = new.children.len();
    let n_old = old.children.len();
    let common_len = n_old.min(n_new);
    let range = 0..common_len;
    for k in range {
        if k != 0 {
            truncates += 1;
            patch.push(PatchItem::NextNode());
        }
        let old_node = old.children.get(k).unwrap();
        let new_node = new.children.get(k).unwrap();
        ret |= diff_recursive(old_node, new_node, patch);
    }

    if n_old > n_new {
        patch.push(PatchItem::TruncateSiblings());
        ret = true;
    } else if n_new > n_old {
        for k in n_old..n_new {
            let new_node = new.children.get(k).unwrap();
            patch.push(PatchItem::AppendSibling(new_node));
            ret = true;
        }
    }

    if ret {
        patch.push(PatchItem::Ascend())
    } else {
        // remove Descend() and NextNode() again
        patch.items.truncate(patch.items.len() - truncates);
    }

    ret
}

fn diff_events<'a>(old: &'a VElement, new: &'a VElement) -> bool {
    if new.events.len() != old.events.len() {
        return false;
    }
    for k in 0..new.events.len() {
        if new.events[k] != old.events[k] {
            return false;
        }
    }
    true
}

fn diff_recursive<'a>(old: &'a VNode, new: &'a VNode, patch: &mut Patch<'a>) -> bool {
    let mut ret = false;
    match (old, new) {
        (VNode::Element(elem_old), VNode::Element(elem_new)) => {
            if elem_old.tag != elem_new.tag
                || elem_old.namespace != elem_new.namespace
                || !diff_events(elem_old, elem_new)
            {
                ret = true;
                patch.push(PatchItem::Replace(new))
            } else {
                ret |= diff_attrs(elem_old, elem_new, patch);
                let _new_id = (*elem_new).id;
                ret |= diff_children(elem_old, elem_new, patch);
                if !elem_old.id.is_empty() {
                    patch.translate(elem_new.id, elem_old.id);
                }
            }
        }
        (VNode::Text(elem_old), VNode::Text(elem_new)) => {
            if elem_old != elem_new {
                ret = true;
                patch.push(PatchItem::ChangeText(elem_new))
            }
        }
        (_, new) => {
            ret = true;
            patch.push(PatchItem::Replace(new));
        },
    };
    ret
}

#[cfg(test)]
mod tests;
