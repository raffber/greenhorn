//! This module implement the DOM diffing algorithm
//!
//! Currently, the algorithm is very simple and perform in O(n).
//! It recursively compares two DOM nodes and diffs the all nodes in
//! the order they are given.
//! Therefore, neither DOM node re-orders nor insertion are detected by this algorithm.
//! In these cases, the DOM will simply be re-emiited.
//!

use crate::runtime::{Frame, RenderResult};
use crate::vdom::{Patch, PatchItem, VElement, VNode};
use crate::App;
use std::collections::HashMap;

// Expansion ideas
// [ ] key based diffing
// Refer to https://programming.vip/docs/realization-and-analysis-of-virtual-dom-diff-algorithm.html
// performance tuning...
// hash based diffing => compute hash in parallel
// https://github.com/Matt-Esch/virtual-dom/blob/master/vtree/diff.js
//
// potentially parallelize using a concurrent hashmap in RenderResult (or just use RwLock<HashMap<>>
//

/// Helper object to diff a previously applied `Frame` with a new
/// result from the `render()` cycle.
pub(crate) struct Differ<'a, A: App> {
    old: &'a Frame<A>,
    new: &'a RenderResult<A>,
}

impl<'a, A: App> Differ<'a, A> {
    pub(crate) fn new(old: &'a Frame<A>, new: &'a RenderResult<A>) -> Self {
        Self { old, new }
    }

    /// Produce a patch based on the RenderResult
    pub(crate) fn diff(self) -> Patch<'a> {
        let mut patch = Patch::new();
        self.diff_recursive(&self.old.rendered.vdom, &self.new.vdom, &mut patch);
        patch.optimize();
        self.diff_blobs(&mut patch);
        patch
    }

    /// Diffs all blobs of the two render results and emits PatchItems accordingly.
    fn diff_blobs(&self, patch: &mut Patch<'a>) {
        for k in self.old.rendered.blobs.keys() {
            if !self.new.blobs.contains_key(k) {
                patch.push(PatchItem::RemoveBlob(*k));
            }
        }
        for (k, v) in &self.new.blobs {
            if let Some(blob) = self.old.rendered.blobs.get(k) {
                if blob.hash() != v.hash() {
                    // NOTE: RemoveBlob is not required, blob will
                    // simply be overwritten
                    patch.push(PatchItem::AddBlob(v.clone()));
                }
            } else {
                // blob missing, add it
                patch.push(PatchItem::AddBlob(v.clone()));
            }
        }
    }

    /// Diffs all attributes of the two elements and emits patches accordingly.
    /// Returns true if changes were detected.
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

    /// Diffs all js events of the two elements and emits patches accordingly.
    /// Returns true if changes were detected.
    fn diff_js_events(&self, old: &'a VElement, new: &'a VElement, patch: &mut Patch<'a>) -> bool {
        let mut ret = false;

        let mut old_kv = HashMap::with_capacity(old.js_events.len());
        for attr in old.js_events.iter() {
            old_kv.insert(&attr.key, &attr.value);
        }
        let mut new_kv = HashMap::with_capacity(new.js_events.len());
        for attr in new.js_events.iter() {
            new_kv.insert(&attr.key, &attr.value);
        }

        for attr in new.js_events.iter() {
            if let Some(&old_v) = old_kv.get(&attr.key) {
                if old_v != &attr.value {
                    ret = true;
                    let p = PatchItem::ReplaceJsEvent(&attr.key, &attr.value);
                    patch.push(p);
                }
            } else {
                ret = true;
                let p = PatchItem::AddJsEvent(&attr.key, &attr.value);
                patch.push(p);
            }
        }

        for attr in old.js_events.iter() {
            if !new_kv.contains_key(&attr.key) {
                ret = true;
                let p = PatchItem::RemoveJsEvent(&attr.key);
                patch.push(p);
            }
        }

        ret
    }

    /// Recursively diffs all children of the elements and emit patches accordingly.
    /// Returns true if changes were detected.
    #[allow(clippy::comparison_chain)]
    fn diff_children(&self, old: &'a VElement, new: &'a VElement, patch: &mut Patch<'a>) -> bool {
        if old.children.is_empty() && new.children.is_empty() {
            return false;
        }

        if !old.children.is_empty() && new.children.is_empty() {
            patch.push(PatchItem::RemoveChildren());
            return true;
        }

        if old.children.is_empty() && !new.children.is_empty() {
            patch.push(PatchItem::AddChildren(&new.children));
            return true;
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
                if let Some(PatchItem::NextNode(skips)) = patch.peek() {
                    patch.pop();
                    patch.push(PatchItem::NextNode(skips + 1))
                } else {
                    truncates += 1;
                    patch.push(PatchItem::NextNode(1));
                }
            }
            let old_node = old.children.get(k).unwrap();
            let new_node = new.children.get(k).unwrap();
            ret |= self.diff_recursive(old_node, new_node, patch);
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

    /// Diffs the registered event handlers and returns true in case
    /// the registered handlers have changed.
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

    /// Recursively diff to vdoms and compute a patch to update `old` to `new`.
    /// Returns whether a change was detected
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
                    ret |= self.diff_js_events(elem_old, elem_new, patch);
                    ret |= self.diff_children(elem_old, elem_new, patch);
                    if !elem_old.id.is_empty() {
                        let very_old_id = self
                            .old
                            .translations
                            .get(&elem_old.id)
                            .unwrap_or(&elem_old.id);
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
            (VNode::Placeholder(id_old, _path_old), VNode::Placeholder(id_new, _path_new)) => {
                if id_old == id_new && !self.new.rendered.contains(id_new) {
                    let new_comp = self.new.get_rendered_component(*id_new).unwrap();
                    for (child_id, child_path) in new_comp.children() {
                        patch.push_path(child_path);
                        let cur_len = patch.len();

                        // new vdom must exist since otherwise we wouldn't see the placeholder
                        let new_vdom = self.new.get_component_vdom(*child_id).unwrap();

                        // old vdom must exist since this component was not re-rendered.
                        // thus it must have the same children
                        let old_vdom = self.old.rendered.get_component_vdom(*child_id).unwrap();

                        self.diff_recursive(old_vdom, new_vdom, patch);

                        // check if a patch was actually emitted
                        if patch.len() == cur_len {
                            // no patch was emitted, thus, reverse the path again
                            patch.pop_path(child_path);
                            ret = false;
                        } else {
                            // some patch was emitted, thus navigate back in DOM
                            patch.push_reverse_path(child_path);
                            ret = true;
                        }
                    }
                } else if id_old == id_new {
                    let old_vdom = self.old.rendered.get_component_vdom(*id_old).unwrap();
                    let new_vdom = self.new.get_component_vdom(*id_new).unwrap();
                    ret = self.diff_recursive(old_vdom, new_vdom, patch);
                } else {
                    // don't even bother diffing
                    let new_vdom = self.new.get_component_vdom(*id_new).unwrap();
                    patch.push(PatchItem::Replace(new_vdom));
                    ret = true;
                }
            }
            (_, new) => {
                ret = true;
                patch.push(PatchItem::Replace(new));
            }
        };
        ret
    }
}
