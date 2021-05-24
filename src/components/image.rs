use crate::blob::Blob;
use crate::node::Node;
use crate::Id;

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

    pub fn mime_type<T: Into<String>>(mut self, mime_type: T) -> Self {
        self.mime_type = mime_type.into();
        self
    }
}

impl Into<Image> for ImageBuilder {
    fn into(self) -> Image {
        let blob_id = Id::new();
        let html_id = self.html_id.unwrap_or_else(|| format!("__id_{}", blob_id));
        let blob = Image::build_blob(&html_id, blob_id, self.data, self.mime_type);
        Image { blob, html_id }
    }
}

impl Image {
    fn build_blob(html_id: &str, blob_id: Id, data: Vec<u8>, mime_type: String) -> Blob {
        let js = format!(
            "{{
            var elem = document.getElementById('{}');
            var blob = app.getBlob({});
            var img_url = URL.createObjectURL(blob.blob);
            elem.src = img_url;
        }}",
            html_id,
            blob_id.data()
        );

        Blob::build(Id::new().id)
            .data(data)
            .mime_type(mime_type)
            .id(blob_id)
            .on_change(js)
            .into()
    }

    pub fn build() -> ImageBuilder {
        ImageBuilder {
            data: Vec::new(),
            mime_type: String::new(),
            html_id: None,
        }
    }

    pub fn update(&mut self, data: Vec<u8>) {
        let blob = Image::build_blob(
            &self.html_id,
            self.blob.id(),
            data,
            self.blob.mime_type().into(),
        );
        self.blob = blob;
    }

    pub fn render<T: 'static + Send>(&self) -> Node<T> {
        let js = format!(
            "{{
            var blob = app.getBlob({});
            var img_url = URL.createObjectURL(blob.blob);
            $event.target.src = img_url;
        }}",
            self.blob.id().data()
        );
        Node::html()
            .elem("img")
            .id(self.html_id.clone())
            .js_event("render", js)
            .add(&self.blob)
            .build()
    }
}
