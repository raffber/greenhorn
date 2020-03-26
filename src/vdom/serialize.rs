use crate::vdom::{EventHandler, Patch, PatchItem, VNode};
use crate::{Id, App};
use crate::runtime::RenderResult;

trait PatchSerialize {
    fn serialize(&self, output: &mut Vec<u8>);
}

trait NodeSerialize {
    fn serialize<A: App>(&self, rendered: &RenderResult<A>, output: &mut Vec<u8>);
}

impl NodeSerialize for VNode {
    fn serialize<A: App>(&self, rendered: &RenderResult<A>, output: &mut Vec<u8>) {
        match self {
            VNode::Element(elem) => {
                output.push(0);
                elem.id.serialize(output);
                elem.tag.serialize(output);
                (elem.attr.len() as u32).serialize(output);
                for attr in elem.attr.iter() {
                    attr.key.serialize(output);
                    attr.value.serialize(output);
                }
                (elem.events.len() as u32).serialize(output);
                for evt in elem.events.iter() {
                    evt.serialize(output);
                }
                (elem.children.len() as u32).serialize(output);
                for c in elem.children.iter() {
                    c.serialize(rendered, output);
                }
                elem.namespace.serialize(output);
            },
            VNode::Text(elem) => {
                output.push(1);
                elem.serialize(output);
            },
            VNode::Placeholder(id) => {
                let vdom = rendered.get_component_vdom(id).unwrap();
                vdom.serialize(rendered, output);
            }
        }
    }
}

impl PatchSerialize for Id {
    fn serialize(&self, output: &mut Vec<u8>) {
        if self.is_empty() {
            output.push(0);
        } else {
            output.push(1);
            output.extend_from_slice(&self.id.to_le_bytes());
        }
    }
}

impl PatchSerialize for u32 {
    fn serialize(&self, output: &mut Vec<u8>) {
        let data = self.to_le_bytes();
        output.extend_from_slice(&data);
    }
}

impl PatchSerialize for u64 {
    fn serialize(&self, output: &mut Vec<u8>) {
        let data = self.to_le_bytes();
        output.extend_from_slice(&data);
    }
}

impl PatchSerialize for Vec<u8> {
    fn serialize(&self, output: &mut Vec<u8>) {
        (self.len() as u32).serialize(output);
        output.extend_from_slice(self);
    }
}

impl PatchSerialize for String {
    fn serialize(&self, output: &mut Vec<u8>) {
        (self.len() as u32).serialize(output);
        output.extend_from_slice(self.as_bytes());
    }
}

impl PatchSerialize for &str {
    fn serialize(&self, output: &mut Vec<u8>) {
        (self.len() as u32).serialize(output);
        output.extend_from_slice(self.as_bytes());
    }
}

impl PatchSerialize for EventHandler {
    fn serialize(&self, output: &mut Vec<u8>) {
        output.push(self.no_propagate.into());
        output.push(self.prevent_default.into());
        self.name.serialize(output);
    }
}

impl<T: PatchSerialize> PatchSerialize for Option<T> {
    fn serialize(&self, output: &mut Vec<u8>) {
        output.push(self.is_some().into());
        if let Some(x) = self {
            x.serialize(output);
        }
    }
}

pub(crate) fn serialize<A: App>(rendered: &RenderResult<A>, patch: &Patch) -> Vec<u8> {
    let mut output = Vec::new();
    for patch in &patch.items {
        match patch {
            PatchItem::AppendSibling(node) => {
                output.push(1);
                node.serialize(rendered, &mut output);
            }
            PatchItem::Replace(node) => {
                output.push(3);
                node.serialize(rendered, &mut output);
            }
            PatchItem::ChangeText(text) => {
                output.push(4);
                text.serialize(&mut output);
            }
            PatchItem::Ascend() => {
                output.push(5);
            }
            PatchItem::Descend() => {
                output.push(6);
            }
            PatchItem::RemoveChildren() => {
                output.push(7);
            }
            PatchItem::TruncateSiblings() => {
                output.push(8);
            }
            PatchItem::NextNode() => {
                output.push(9);
            }
            PatchItem::RemoveAttribute(key) => {
                output.push(10);
                key.serialize(&mut output);
            }
            PatchItem::AddAtrribute(key, value) => {
                output.push(11);
                key.serialize(&mut output);
                value.serialize(&mut output);
            }
            PatchItem::ReplaceAttribute(key, value) => {
                output.push(12);
                key.serialize(&mut output);
                value.serialize(&mut output);
            }
            PatchItem::AddBlob(blob) => {
                output.push(13);
                blob.id().serialize(&mut output);
                blob.hash().serialize(&mut output);
                blob.mime_type().serialize(&mut output);
                blob.data().serialize(&mut output);
            }
            PatchItem::RemoveBlob(blob_id) => {
                output.push(14);
                blob_id.serialize(&mut output);
            }
            PatchItem::RemoveJsEvent(key) => {
                output.push(15);
                key.serialize(&mut output);
            }
            PatchItem::AddJsEvent(key, value) => {
                output.push(16);
                key.serialize(&mut output);
                value.serialize(&mut output);
            }
            PatchItem::ReplaceJsEvent(key, value) => {
                output.push(17);
                key.serialize(&mut output);
                value.serialize(&mut output);
            }
            PatchItem::AddChildren(children) => {
                output.push(18);
                (children.len() as u32).serialize(&mut output);
                for child in *children {
                    child.serialize(rendered, &mut output);
                }
            }
        }
    }
    output
}
