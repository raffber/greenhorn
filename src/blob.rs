use crate::node_builder::BlobBuilder;
use std::fmt::{Debug, Formatter, Error};
use std::sync::Arc;
use crate::Id;

/// keeps track of all the blob data. `BlobData` objects are
/// shared between serveral blobs.
/// Since they are wrapped in `Arc<...>`, blobs are immutable.
pub(crate) struct BlobData {
    hash: u64,
    id: Id,
    data: Vec<u8>,
    mime_type: String,
    on_change: Option<String>,
    on_add: Option<String>
}

///
/// A `Blob` corresponds to some binary data,
///
/// `Blob` objects have an assoociated `id` and `hash` to make them comparable.
/// The user of this type is responsible for providing unique `id` and `hash` value combinations.
///
/// The `id` field is supposed to identify the purpose of the `Blob` within the application.
/// For example, it might be used to point to an image in the frontend.
///
/// The `hash` field should reflect the content of the `Blob`.
/// It might also be considered the version of the underlying data.
///
/// Associated to the `Blob` a MIME-type can be provided. This is useful meta information for
/// media files or similar.
///
/// ## Example
///
/// ```
/// // TODO ...
/// ```
///
#[derive(Clone)]
pub struct Blob {
    pub(crate) inner: Arc<BlobData>
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
        self.inner.on_change.as_deref()
    }

    pub fn on_add(&self) -> Option<&str> {
        self.inner.on_add.as_deref()
    }
}

impl PartialEq for Blob {
    fn eq(&self, other: &Self) -> bool {
        self.inner.hash == other.inner.hash && self.inner.id == other.inner.id
    }
}

impl Debug for Blob {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str(&format!("Blob {{id: {}, hash: {}}}", self.id(), self.hash()))
    }
}

impl From<BlobBuilder> for Blob {
    fn from(builder: BlobBuilder) -> Self {
        let id = builder.id.unwrap_or_else( Id::new);
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

