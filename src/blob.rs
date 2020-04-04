use crate::node_builder::{AddNodes, BlobBuilder};
use std::iter::{Once, once};
use crate::node::Node;
use std::fmt::{Debug, Formatter, Error};
use std::sync::Arc;
use crate::Id;

pub struct BlobData {
    hash: u64,
    id: Id,
    data: Vec<u8>,
    mime_type: String,
}

#[derive(Clone)]
pub struct Blob {
    inner: Arc<BlobData>
}

impl Blob {
    pub fn build(id: Id, hash: u64) -> BlobBuilder {
        BlobBuilder {
            id,
            hash,
            mime_type: "".to_string(),
            data: vec![]
        }
    }

    pub fn id(&self) -> Id {
        self.inner.id
    }

    pub fn hash(&self) -> u64 {
        self.inner.hash
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.inner.data
    }

    pub fn mime_type(&self) -> &str {
        &self.inner.mime_type
    }
}

impl Debug for Blob {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str(&format!("<Blob id={} hash={}>", self.id(), self.hash()))
    }
}

impl From<BlobBuilder> for Blob {
    fn from(builder: BlobBuilder) -> Self {
        Blob {
            inner: Arc::new(BlobData {
                hash: builder.hash,
                id: builder.id,
                data: builder.data,
                mime_type: builder.mime_type
            }),
        }
    }
}

impl<T: 'static> From<Blob> for Node<T> {
    fn from(blob: Blob) -> Self {
        Node::Blob(blob)
    }
}

impl<T: 'static> AddNodes<T> for Blob {
    type Output = Once<Node<T>>;

    fn into_nodes(self) -> Self::Output {
        once(Node::Blob(self))
    }
}