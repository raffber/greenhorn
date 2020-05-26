use crate::dialog::Dialog;
use serde::{Deserialize, Serialize};

/// Represents a filter in a [`FileOpenDialog`](struct.FileOpenDialog.html) or a [`FileSaveDialog`](struct.FileSaveDialog.html).
/// A filter consists of a name and a list of file extensions (such as "js" or "rs").
#[derive(Serialize, Deserialize)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

impl FileFilter {
    /// Create a new `FileFilter` with the given description.
    pub fn new<T: Into<String>>(description: T) -> Self {
        Self {
            name: description.into(),
            extensions: vec![],
        }
    }

    /// Create a new file filter with a description and a list of filters
    pub fn new_with_extensions<T: Into<String>>(description: T, filters: Vec<String>) -> Self {
        Self {
            name: description.into(),
            extensions: filters,
        }
    }

    /// Append a new extension to the `FileFilter`.
    pub fn push<T: Into<String>>(mut self, filter: T) -> Self {
        let filter = filter.into();
        self.extensions.push(filter);
        self
    }
}

/// Message emitted when closing a [`FileOpenDialog`](struct.FileOpenDialog.html).
#[derive(Debug, Serialize, Deserialize)]
pub enum FileOpenMsg {
    Canceled,
    Selected(String),
    SelectedMultiple(Vec<String>),
}

/// Represents a system dialog for opening one or multiple files.
/// Once the dialog is closed it resolves to a [`FileOpenMsg`](enum.FileOpenMsg.html).
///
/// # Example
///
/// ```
/// # use greenhorn::context::Context;
/// # use greenhorn::dialog::{FileOpenMsg, FileOpenDialog, FileFilter};
/// #
/// enum Msg {
///     FileOpened(FileOpenMsg),
///     LoadFile,
/// }
///
/// fn update(msg: Msg, ctx: Context<Msg>) {
///     match msg {
///         Msg::FileOpened(msg) => {
///             match msg {
///                 FileOpenMsg::Canceled => {println!("dialog was canceled!")}
///                 FileOpenMsg::Selected(_fpath) => {println!("A single item  was selected!")}
///                 FileOpenMsg::SelectedMultiple(_fpaths) => {println!("Multiple items were selected!")}
///             }
///         }
///
///         Msg::LoadFile => {
///             let dialog = FileOpenDialog::new("Open a File", "~")
///                 .with_filter(FileFilter::new("Rust files").push("rs"))
///                 .with_filter(FileFilter::new("Javascript files").push("js").push("jsx"))
///                 .allow_multiple();
///             ctx.dialog(dialog, Msg::FileOpened);
///         }
///     }
/// }
/// ```
///
#[derive(Serialize, Deserialize)]
pub struct FileOpenDialog {
    pub filter: Vec<FileFilter>,
    pub multiple: bool,
    pub title: String,
    pub path: String,
}

impl FileOpenDialog {
    /// Create a new `FileOpenDialog`.
    /// The `title` argument is displayed in the window title bar.
    /// The `path` arguments defines the initial file path.
    pub fn new<A: Into<String>, B: Into<String>>(title: A, path: B) -> Self {
        Self {
            filter: Vec::new(),
            multiple: false,
            title: title.into(),
            path: path.into(),
        }
    }

    /// Add a file filter to this `FileOpenDialog`
    pub fn with_filter(mut self, filter: FileFilter) -> Self {
        self.filter.push(filter);
        self
    }

    /// Allow the multiple files to be selected. If multiple files are selected,
    /// the dialog resolves to a `FileOpenMsg::SelectedMultiple` message.
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

/// Message type a [`FileSaveDialog`](struct.FileSaveDialog.html) resolves to when closed.
#[derive(Debug, Serialize, Deserialize)]
pub enum FileSaveMsg {
    SaveTo(String),
    Canceled,
}


/// Represents a system dialog for saving a file.
/// Once the dialog is closed it resolves to a [`FileSaveMsg`](enum.FileSaveMsg.html).
///
/// # Example
///
/// ```
/// # use greenhorn::context::Context;
/// # use greenhorn::dialog::{FileFilter, FileSaveDialog, FileSaveMsg};
/// #
/// enum Msg {
///     FileSaved(FileSaveMsg),
///     SaveAsClicked,
/// }
///
/// fn update(msg: Msg, ctx: Context<Msg>) {
///     match msg {
///         Msg::FileSaved(msg) => {
///             match msg {
///                 FileSaveMsg::Canceled => {println!("Dialog was canceled!")}
///                 FileSaveMsg::SaveTo(fpath) => {println!("Save to {}", fpath)}
///             }
///         }
///
///         Msg::SaveAsClicked => {
///             let dialog = FileSaveDialog::new("Save As", "~")
///                 .with_filter(FileFilter::new("Rust files").push("rs"))
///                 .with_filter(FileFilter::new("Javascript files").push("js").push("jsx"));
///             ctx.dialog(dialog, Msg::FileSaved);
///         }
///     }
/// }
/// ```
///
#[derive(Serialize, Deserialize)]
pub struct FileSaveDialog {
    pub title: String,
    pub path: String,
    pub filter: Vec<FileFilter>,
}

impl FileSaveDialog {
    /// Create a new `FileSaveDialog` with a given window `title`. The dialog
    /// opens at the given `path`.
    pub fn new<A: Into<String>, B: Into<String>>(title: A, path: B) -> Self {
        Self {
            filter: Vec::new(),
            title: title.into(),
            path: path.into(),
        }
    }

    /// Add a [`FileFilter`](struct.FileFilter.html) to the `FileSaveDialog`.
    pub fn with_filter(mut self, filter: FileFilter) -> Self {
        self.filter.push(filter);
        self
    }
}

impl Dialog for FileSaveDialog {
    type Msg = FileSaveMsg;

    fn type_name() -> &'static str {
        "FileSaveDialog"
    }
}
