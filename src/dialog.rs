use serde::{Serialize, Deserialize};
use std::marker::PhantomData;
use std::sync::{Mutex, Arc};

mod private {
    use super::*;

    pub trait Sealed {}

    impl Sealed for FileOpenDialog {}
    impl Sealed for MultipleFileOpenDialog {}
    impl Sealed for MessageBox {}
    impl Sealed for FileSaveDialog {}
}


trait DialogMsg<'de> : Serialize + Deserialize<'de> {}

pub trait Dialog<'de>: private::Sealed + Serialize + Deserialize<'de> {
    type Msg: DialogMsg<'de>;

    /// Called by the runtime to produce a result based on the received
    /// data from the dialog after the user has closed it.
    fn resolve(self, data: &'de str) -> Result<Self::Msg, serde_json::Error> {
        serde_json::from_str(data)
    }
}


#[derive(Serialize, Deserialize)]
pub enum FileOpenMsg {
    Cancel(),
    Selected(String),
}

impl DialogMsg<'_> for FileOpenMsg {}

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

impl DialogMsg<'_> for MultipleFileOpenMsg {}

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

impl DialogMsg<'_> for MessageBoxMsg {}

#[derive(Serialize, Deserialize)]
pub struct MessageBox {

}

impl Dialog<'_> for MessageBox {
    type Msg = MessageBoxMsg;
}


#[derive(Serialize, Deserialize)]
pub enum FileSaveMsg {
    SaveTo(String),
    Cancel()
}

impl DialogMsg<'_> for FileSaveMsg {}


#[derive(Serialize, Deserialize)]
pub struct FileSaveDialog {

}

impl Dialog<'_> for FileSaveDialog {
    type Msg = FileSaveMsg;
}


pub(crate) struct DialogBinding<T: Send + 'static> {
    phantom: PhantomData<T>,
}

impl<T: Send + 'static> DialogBinding<T> {
    pub(crate) fn new<'de, D: Dialog<'de>, F: Fn(D::Msg) -> T>(&self, _dialog: D, _fun: F) -> Self {
        todo!()
    }
}

trait DialogBindingTrait<T: Send + 'static> {
    fn resolve(self, data: &str) -> Result<T, serde_json::Error>;
}

struct DialogBindingDirect<'de, T: Send + 'static, U: Dialog<'de>, Fun: Fn(U::Msg) -> T> {
    fun: Arc<Mutex<Fun>>,
    dialog: U,
    marker: PhantomData<&'de T>,
}

struct DialogBindingMap<'de, T: Send + 'static, U: Send + 'static, Fun: Fn(U) -> T> {
    fun: Arc<Mutex<Fun>>,
    inner: Box<dyn DialogBindingTrait<U>>,
    marker: PhantomData<&'de T>,
}

impl<'de, T: Send + 'static, U: Dialog<'de>, Fun: Fn(U::Msg) -> T> DialogBindingTrait<T> for DialogBindingDirect<'de, T, U, Fun> {
    fn resolve(self, data: &str) -> Result<T, serde_json::Error> {
        let msg = self.dialog.resolve(data)?;
        let fun = self.fun.lock().unwrap();
        let ret = (*fun)(msg);
        Ok(ret)
    }
}

impl<'de, T: Send + 'static, U: Send + 'static, Fun: Fn(U) -> T> DialogBindingTrait<T> for DialogBindingMap<'de, T, U, Fun> {
    fn resolve(self, data: &str) -> Result<T, serde_json::Error> {
        let msg = self.inner.resolve(data)?;
        let fun = self.fun.lock().unwrap();
        let ret = (*fun)(msg);
        Ok(ret)
    }
}

