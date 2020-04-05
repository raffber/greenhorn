use crate::node_builder::{AddNodes, BlobBuilder};
use std::iter::{Once, once};
use crate::node::Node;
use std::fmt::{Debug, Formatter, Error};
use std::sync::Arc;
use crate::Id;
use std::ops::Deref;

pub struct BlobData {
    hash: u64,
    id: Id,
    data: Vec<u8>,
    mime_type: String,
    on_change: Option<String>,
    on_add: Option<String>
}

#[derive(Clone)]
pub struct Blob {
    inner: Arc<BlobData>
}

impl Blob {
    pub fn build(hash: u64) -> BlobBuilder {
        BlobBuilder {
            id: None,
            hash,
            mime_type: "".to_string(),
            data: vec![],
            on_change: None,
            on_add: None
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

    pub fn on_change(&self) -> Option<&str> {
        self.inner.on_change.as_ref().map(|x| x.deref())
    }

    pub fn on_add(&self) -> Option<&str> {
        self.inner.on_add.as_ref().map(|x| x.deref())
    }
}

impl Debug for Blob {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str(&format!("Blob {{id: {}, hash: {}}}", self.id(), self.hash()))
    }
}

impl From<BlobBuilder> for Blob {
    fn from(builder: BlobBuilder) -> Self {
        let id = builder.id.unwrap_or_else(|| Id::new());
        Blob {
            inner: Arc::new(BlobData {
                hash: builder.hash,
                id,
                data: builder.data,
                mime_type: builder.mime_type,
                on_change: builder.on_change,
                on_add: builder.on_add,
            }),
        }
    }
}

impl<T: 'static> AddNodes<T> for &Blob {
    type Output = Once<Node<T>>;

    fn into_nodes(self) -> Self::Output {
        once(Node::Blob(Blob {
            inner: self.inner.clone()
        }))
    }
}