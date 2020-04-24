use crate::dialog::Dialog;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FileFilter {
    pub description: String,
    pub filters: Vec<String>,
}

impl FileFilter {
    pub fn new<T: Into<String>>(description: T) -> Self {
        Self {
            description: description.into(),
            filters: vec![],
        }
    }

    pub fn new_from_multiple<T: Into<String>>(description: T, filters: Vec<String>) -> Self {
        Self {
            description: description.into(),
            filters,
        }
    }

    pub fn push<T: Into<String>>(mut self, filter: T) -> Self {
        let filter = filter.into();
        self.filters.push(filter);
        self
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FileOpenMsg {
    Canceled,
    Selected(String),
    SelectedMultiple(Vec<String>),
}

#[derive(Serialize, Deserialize)]
pub struct FileOpenDialog {
    pub filter: Option<FileFilter>,
    pub multiple: bool,
    pub title: String,
    pub path: String,
}

impl FileOpenDialog {
    pub fn new<A: Into<String>, B: Into<String>>(title: A, path: B) -> Self {
        Self {
            filter: None,
            multiple: false,
            title: title.into(),
            path: path.into(),
        }
    }

    pub fn with_filter(mut self, filter: FileFilter) -> Self {
        self.filter = Some(filter);
        self
    }

    pub fn allow_multiple(mut self) -> Self {
        self.multiple = true;
        self
    }
}

impl Dialog for FileOpenDialog {
    type Msg = FileOpenMsg;

    fn type_name() -> &'static str {
        "FileOpenDialog"
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FileSaveMsg {
    SaveTo(String),
    Cancel,
}

#[derive(Serialize, Deserialize)]
pub struct FileSaveDialog {
    pub title: String,
    pub path: String,
    pub filter: Option<FileFilter>,
}

impl FileSaveDialog {
    pub fn new<A: Into<String>, B: Into<String>>(title: A, path: B) -> Self {
        Self {
            filter: None,
            title: title.into(),
            path: path.into(),
        }
    }

    pub fn with_filter(mut self, filter: FileFilter) -> Self {
        self.filter = Some(filter);
        self
    }
}

impl Dialog for FileSaveDialog {
    type Msg = FileSaveMsg;

    fn type_name() -> &'static str {
        "FileSaveDialog"
    }
}
