use serde::{Serialize, Deserialize};

mod private {
    use super::*;

    pub trait Sealed {}

    impl Sealed for FileOpenDialog {}
    impl Sealed for MultipleFileOpenDialog {}
    impl Sealed for MessageBox {}
}

pub trait Dialog<'de>: private::Sealed + Serialize + Deserialize<'de> {
    type Msg: Serialize + Deserialize<'de>;

}


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

impl Dialog<'_> for FileOpenDialog {
    type Msg = FileOpenMsg;
}

#[derive(Serialize, Deserialize)]
pub enum MultipleFileOpenMsg {
    Cancel(),
    Selected(Vec<String>)
}


#[derive(Serialize, Deserialize)]
pub struct MultipleFileOpenDialog {}

impl Dialog<'_> for MultipleFileOpenDialog {
    type Msg = MultipleFileOpenMsg;

}

#[derive(Serialize, Deserialize)]
pub enum MessageBoxMsg {
    Ok(),
    Cancel(),
    Yes(),
    No(),
}

#[derive(Serialize, Deserialize)]
pub struct MessageBox {

}

impl Dialog<'_> for MessageBox {
    type Msg = MessageBoxMsg;
}



