use crate::{App, Id};
use crate::runtime::{RenderResult, Frame};
use crate::vdom::{Patch, VElement, PatchItem, VNode};
use std::collections::{HashMap, HashSet};

pub(crate) struct Differ<'a, A: App> {
    old: &'a Frame<A>,
    new: &'a RenderResult<A>,
    rendered: HashSet<Id>,
}

impl<'a, A: App> Differ<'a, A> {
    pub(crate) fn new(old: &'a Frame<A>, new: &'a RenderResult<A>, rendered: HashSet<Id>) -> Self {
        Self { old, new, rendered }
    }

    pub(crate) fn diff(&self) -> Patch<'a> {
        let mut patch = Patch::new();
        self.diff_recursive(&self.old.rendered.root,   &self.new.root, &mut patch);
        patch.optimize();
        patch
    }

    fn diff_attrs(&self, old: &'a VElement, new: &'a VElement, patch: &mut Patch<'a>) -> bool {
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
    fn diff_children(&self, old: &'a VElement, new: &'a VElement, patch: &mut Patch<'a>) -> bool {
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
            ret |= self.diff_recursive(old_node,  new_node, patch);
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

    fn diff_events(&self, old: &'a VElement, new: &'a VElement) -> bool {
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

    fn diff_recursive(&self, old: &'a VNode, new: &'a VNode, patch: &mut Patch<'a>) -> bool {
        let mut ret = false;
        match (old, new) {
            (VNode::Element(elem_old), VNode::Element(elem_new)) => {
                if elem_old.tag != elem_new.tag
                    || elem_old.namespace != elem_new.namespace
                    || !self.diff_events(elem_old, elem_new)
                {
                    ret = true;
                    patch.push(PatchItem::Replace(new))
                } else {
                    ret |= self.diff_attrs(elem_old, elem_new, patch);
                    let _new_id = (*elem_new).id;
                    ret |= self.diff_children(elem_old, elem_new, patch);
                    if !elem_old.id.is_empty() {
                        let very_old_id = self.old.translations.get(&elem_old.id).unwrap_or(&elem_old.id);
                        patch.translate(elem_new.id, *very_old_id);
                    }
                }
            }
            (VNode::Text(elem_old), VNode::Text(elem_new)) => {
                if elem_old != elem_new {
                    ret = true;
                    patch.push(PatchItem::ChangeText(elem_new))
                }
            }
            (VNode::Placeholder(id_old), VNode::Placeholder(id_new)) => {
                if id_old == id_new && !self.rendered.contains(id_new) {
                    // TODO: skip this component, navigate to its children and diff those if they
                    // were rendered
                    let old_vdom = self.old.rendered.get_component_vdom(id_old).unwrap();
                    let new_vdom = self.new.get_component_vdom(id_new).unwrap();
                    self.diff_recursive(old_vdom,  new_vdom, patch);
                } else if id_old == id_new {
                    let old_vdom = self.old.rendered.get_component_vdom(id_old).unwrap();
                    let new_vdom = self.new.get_component_vdom(id_new).unwrap();
                    self.diff_recursive(old_vdom,  new_vdom, patch);
                } else {
                    // don't even bother diffing
                    let new_vdom = self.new.get_component_vdom(id_new).unwrap();
                    patch.push(PatchItem::Replace(new_vdom));
                }
            },
            (_, new) => {
                ret = true;
                patch.push(PatchItem::Replace(new));
            },
        };
        ret
    }
}

