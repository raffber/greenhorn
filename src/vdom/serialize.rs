use crate::vdom::{Patch, PatchItem, VNode, EventHandler};
use crate::Id;

trait PatchSerialize {
    fn serialize(&self, output: &mut Vec<u8>);
}

impl PatchSerialize for VNode {
    fn serialize(&self, output: &mut Vec<u8>) {
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
                    c.serialize(output);
                }
            }
            VNode::Text(elem) => {
                output.push(1);
                elem.serialize(output);
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
        self.iter().for_each(|x| x.serialize(output));
    }
}

pub fn serialize(patch: Patch) -> Vec<u8> {
    let mut output = Vec::new();
    for patch in patch.items {
        match patch {
            PatchItem::AppendNode(node) => {
                output.push(1);
                node.serialize(&mut output);
            }
            PatchItem::Replace(node) => {
                output.push(3);
                node.serialize(&mut output);
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
            PatchItem::TruncateNodes() => {
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
        }
    }
    output
}
