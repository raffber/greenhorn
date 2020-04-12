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
/// `Blob`s allow transferring binary data from backend to frontend.
///
/// `Blob` objects have an assoociated `id` and `hash` fields to make them comparable and identifiable.
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
/// Blobs require manual javascript interaction on the frontend side. Two hooks are provided:
///  * `on_change()`: Called whenever the `hash` of a Blob changes, but the `id` remains the same
///  * `on_add()`: Called when at least the `id` changes
///
/// ## Example
///
/// ```
/// # use greenhorn::blob::Blob;
/// # use greenhorn::Render;
/// # use greenhorn::node::Node;
///
/// struct MyImage {
///     blob: Blob,
/// }
///
/// impl MyImage {
///     fn new() -> Self {
///         // hash can be constant, if we don't plan to change the content
///         let blob = Blob::build(0)
///             .mime_type("image/png")
///             .data(vec![1,2,3])      // add image data here!
///             .build();
///         MyImage { blob }
///     }
/// }
///
/// impl Render for MyImage {
///     type Message = ();
///
///     fn render(&self) -> Node<Self::Message> {
///         let js = format!("{{
///             var blob = app.getBlob({});
///             var img_url = URL.createObjectURL(blob.blob);
///             event.target.src = img_url;
///         }}", self.blob.id().data());
///         Node::html().elem("img")
///             .js_event("render", js)
///             .add(&self.blob)
///             .build()
///     }
/// }
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

    /// Returns the id associated with this blob.
    ///
    /// Note that the id and the hash define a unique key to the `Blob`.
    /// The `id` should define the blob within the application data-flow and is usually
    /// auto-generated.
    pub fn id(&self) -> Id {
        self.inner.id
    }

    /// Returns the hash of the data associated with this blob.
    ///
    /// Note that the id and the hash define a unique key to the `Blob`.
    /// The `hash` should represent the version of the data underlying the `Blob`.
    /// Thus, if the `hash` changes, the `Blob` is considered changed and re-transmitted to
    /// the frontend.
    pub fn hash(&self) -> u64 {
        self.inner.hash
    }

    /// Returns the data underlying the Blob
    pub fn data(&self) -> &Vec<u8> {
        &self.inner.data
    }

    /// Returns the mime-type of the data
    pub fn mime_type(&self) -> &str {
        &self.inner.mime_type
    }

    /// Returns the registered javascript function which is called if the content of
    /// the `Blob` changes, i.e. if the hash of the `Blob` changes but the `Id` remains the same.
    ///
    /// ## Example
    ///
    /// ```
    /// # use greenhorn::node::Node;
    /// # use greenhorn::Id;
    /// # use greenhorn::blob::Blob;
    ///
    /// fn render_blob() -> Blob {
    ///     let blob_id = Id::new();
    ///     let js = format!("{{
    ///         var elem = document.getElementById('my-html-id');
    ///         var blob = app.getBlob({});
    ///         var img_url = URL.createObjectURL(blob.blob);
    ///         elem.src = img_url;
    ///     }}", blob_id.data());
    ///
    ///     Blob::build(0)
    ///         .data(vec![1,2,3])
    ///         .mime_type("image/png")
    ///         .id(blob_id)
    ///         .on_change(js)
    ///         .into()
    /// }
    /// ```
    pub fn on_change(&self) -> Option<&str> {
        self.inner.on_change.as_deref()
    }

    /// Returns the registered javascript function which is called when the `Blob` is
    /// added to the frontend.
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

