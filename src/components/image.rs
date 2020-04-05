use crate::Id;
use crate::node::Node;
use crate::blob::Blob;

pub struct Image {
    blob: Blob,
    html_id: String,
}

pub struct ImageBuilder {
    data: Vec<u8>,
    mime_type: String,
    html_id: Option<String>,
}

impl ImageBuilder {
    pub fn id<T: Into<String>>(mut self, id: T) -> Self {
        self.html_id = Some(id.into());
        self
    }

    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    pub fn build(self) -> Image {
        let hash = Id::new().id;
        let blob: Blob = Blob::build(hash).data(self.data).mime_type(self.mime_type).into();
        let html_id = self.html_id.unwrap_or_else(|| format!("__id_{}", blob.id().data()));
        Image {
            blob,
            html_id
        }
    }
}

impl Into<Image> for ImageBuilder {
    fn into(self) -> Image {
        self.build()
    }
}

impl Image {
    pub fn build<T: Into<String>>(mime_type: T, img: Vec<u8>) -> ImageBuilder {
        ImageBuilder {
            data: img,
            mime_type: mime_type.into(),
            html_id: None,
        }
    }

    pub fn render<T>(&self) -> Node<T> {
        let js = format!("{{
            var blob = app.getBlob({});
            var img_url = URL.createObjectURL(blob.blob);
            event.target.src = img_url;
        }}", self.blob.id().data());
        Node::html().elem("img")
            .id(self.html_id.clone())
            .js_event("render", js )
            .add(&self.blob)
            .build()
    }
}

