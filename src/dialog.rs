use std::marker::PhantomData;
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, Arc};
use serde::de::DeserializeOwned;

mod private {
    use super::*;

    pub trait Sealed {}

    impl Sealed for FileOpenDialog {}
    impl Sealed for MultipleFileOpenDialog {}
    impl Sealed for MessageBox {}
    impl Sealed for FileSaveDialog {}
}

pub trait Dialog: private::Sealed + Serialize + DeserializeOwned + std::marker::Sized {
    type Msg: DeserializeOwned;

    /// Called by the runtime to produce a result based on the received
    /// data from the dialog after the user has closed it.
    fn resolve(self, data: &str) -> Result<Self::Msg, serde_json::Error> {
        let bytes = data.as_bytes();
        serde_json::from_reader(bytes)
    }
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

impl Dialog for FileOpenDialog {
    type Msg = FileOpenMsg;
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
}

#[derive(Serialize, Deserialize)]
pub enum MessageBoxMsg {
    Ok(),
    Cancel(),
    Yes(),
    No(),
}

#[derive(Serialize, Deserialize)]
pub struct MessageBox {}

impl Dialog for MessageBox {
    type Msg = MessageBoxMsg;
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
}

pub(crate) struct DialogBinding<T: Send + 'static> {
    inner: Option<Box<dyn DialogBindingTrait<T>>>,
}

impl<T: Send + 'static> DialogBinding<T> {
    pub(crate) fn new<D: 'static + Dialog, F: 'static + Fn(D::Msg) -> T>(&self, dialog: D, fun: F) -> Self {
        Self {
            inner: Some(Box::new(DialogBindingDirect {
                fun: Arc::new(Mutex::new(fun)),
                dialog: Some(dialog),
                marker: PhantomData
            }))
        }
    }

    pub(crate) fn resolve(mut self, data: &str) -> Result<T, serde_json::Error> {
        self.inner.take().unwrap().resolve(data)
    }

    pub(crate) fn map<U: 'static + Send, F: 'static + Fn(T) -> U>(self, fun: Arc<Mutex<F>>) -> DialogBinding<U> {
        let inner: Box<dyn DialogBindingTrait<U>> = Box::new(DialogBindingMap{
            fun,
            inner: self.inner,
            marker: PhantomData
        });
        DialogBinding { inner: Some(inner) }
    }
}

pub(crate) trait DialogBindingTrait<T: Send + 'static> {
    fn resolve(&mut self, data: &str) -> Result<T, serde_json::Error>;
}

struct DialogBindingDirect<T: Send + 'static, U: Dialog, Fun: Fn(U::Msg) -> T> {
    fun: Arc<Mutex<Fun>>,
    dialog: Option<U>,
    marker: PhantomData<T>,
}

impl<T: Send + 'static, U: Dialog, Fun: Fn(U::Msg) -> T> DialogBindingTrait<T> for DialogBindingDirect<T, U, Fun> {
    fn resolve(&mut self, data: &str) -> Result<T, serde_json::Error> {
        let msg = self.dialog.take().unwrap().resolve(data)?;
        let fun = self.fun.lock().unwrap();
        let ret = (*fun)(msg);
        Ok(ret)
    }
}

struct DialogBindingMap<T: Send + 'static, U: Send + 'static, Fun: Fn(U) -> T> {
    fun: Arc<Mutex<Fun>>,
    inner: Option<Box<dyn DialogBindingTrait<U>>>,
    marker: PhantomData<T>,
}

impl<T: Send + 'static, U: Send + 'static, Fun: Fn(U) -> T> DialogBindingTrait<T> for DialogBindingMap<T, U, Fun> {
    fn resolve(&mut self, data: &str) -> Result<T, serde_json::Error> {
        let msg = self.inner.take().unwrap().resolve(data)?;
        let fun = self.fun.lock().unwrap();
        let ret = (*fun)(msg);
        Ok(ret)
    }
}

