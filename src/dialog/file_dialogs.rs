use serde::{Serialize, Deserialize};
use crate::dialog::Dialog;

#[derive(Serialize, Deserialize)]
pub enum FileOpenMsg {
    Cancel(),
    Selected(String),
}

#[derive(Serialize, Deserialize)]
pub struct FileOpenDialog {}

impl FileOpenDialog {
    fn new() -> Self {
        todo!()
    }

}

impl Dialog for FileOpenDialog {
    type Msg = FileOpenMsg;

    fn type_name() -> &'static str {
        "FileOpenDialog"
    }
}

#[derive(Serialize, Deserialize)]
pub enum MultipleFileOpenMsg {
    Cancel(),
    Selected(Vec<String>)
}


#[derive(Serialize, Deserialize)]
pub struct MultipleFileOpenDialog {}

impl Dialog for MultipleFileOpenDialog {
    type Msg = MultipleFileOpenMsg;

    fn type_name() -> &'static str {
        "MultipleFileOpenDialog"
    }
}

#[derive(Serialize, Deserialize)]
pub enum FileSaveMsg {
    SaveTo(String),
    Cancel()
}

#[derive(Serialize, Deserialize)]
pub struct FileSaveDialog {

}

impl Dialog for FileSaveDialog {
    type Msg = FileSaveMsg;

    fn type_name() -> &'static str {
        "FileSaveDialog"
    }
}
