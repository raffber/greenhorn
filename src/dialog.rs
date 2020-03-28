use serde::{Serialize, Deserialize};
use std::marker::PhantomData;

mod private {
    use super::*;

    pub trait Sealed {}

    impl Sealed for FileOpenDialog {}
    impl Sealed for MultipleFileOpenDialog {}
    impl Sealed for MessageBox {}
    impl Sealed for FileSaveDialog {}
}

pub trait Dialog<'de>: private::Sealed + Serialize + Deserialize<'de> {
    type Msg: Serialize + Deserialize<'de>;

    /// Called by the runtime to produce a result based on the received
    /// data from the dialog after the user has closed it.
    fn resolve(self, data: &str) -> Self::Msg;
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

    fn resolve(self, data: &str) -> Self::Msg {
        unimplemented!()
    }
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

    fn resolve(self, data: &str) -> Self::Msg {
        unimplemented!()
    }
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

    fn resolve(self, data: &str) -> Self::Msg {
        unimplemented!()
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

impl Dialog<'_> for FileSaveDialog {
    type Msg = FileSaveMsg;

    fn resolve(self, data: &str) -> Self::Msg {
        unimplemented!()
    }
}


pub(crate) struct DialogBinding<T: Send + 'static> {
    phantom: PhantomData<T>,
}

impl<T: Send + 'static> DialogBinding<T> {
    pub(crate) fn new<'de, D: Dialog<'de>, F: Fn(D::Msg) -> T>(&self, _dialog: D, _fun: F) -> Self {
        todo!()
    }
}


