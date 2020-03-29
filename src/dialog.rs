use serde::{Serialize, Deserialize};
use std::marker::PhantomData;
use std::sync::{Mutex, Arc};

// TODO: cleanup 'de livetime acc. https://serde.rs/lifetimes.html

mod private {
    use super::*;

    pub trait Sealed {}

    impl Sealed for FileOpenDialog {}
    impl Sealed for MultipleFileOpenDialog {}
    impl Sealed for MessageBox {}
    impl Sealed for FileSaveDialog {}
}


pub trait DialogMsg<'de> : Serialize + Deserialize<'de> {}

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


pub(crate) struct DialogBinding<'de, T: Send + 'static> {
    inner: Option<Box<dyn DialogBindingTrait<'de, T>>>,
}

impl<'de, T: Send + 'static> DialogBinding<'de, T> {
    pub(crate) fn new<D: Dialog<'de>, F: Fn(D::Msg) -> T>(&self, _dialog: D, _fun: F) -> Self {
        todo!()
    }

    pub(crate) fn resolve(mut self, data: &'de str) -> Result<T, serde_json::Error> {
        self.inner.take().unwrap().resolve(data)
    }

    // pub(crate) fn map<U: 'static + Send, F: Fn(T) -> U>(self, fun: Arc<Mutex<F>>) -> DialogBinding<'de, U> {
    //     let inner: Box<dyn DialogBindingTrait<'de, U>> = Box::new(DialogBindingMap{
    //         fun,
    //         inner: self.inner,
    //         marker: PhantomData
    //     });
    //     DialogBinding { inner: Some(inner) }
    // }
}

pub(crate) trait DialogBindingTrait<'de, T: Send + 'static> {
    fn resolve(&mut self, data: &'de str) -> Result<T, serde_json::Error>;
}

struct DialogBindingDirect<'de, T: Send + 'static, U: Dialog<'de>, Fun: Fn(U::Msg) -> T> {
    fun: Arc<Mutex<Fun>>,
    dialog: Option<U>,
    marker: PhantomData<&'de T>,
}

impl<'de, T: Send + 'static, U: Dialog<'de>, Fun: Fn(U::Msg) -> T> DialogBindingTrait<'de, T> for DialogBindingDirect<'de, T, U, Fun> {
    fn resolve(&mut self, data: &'de str) -> Result<T, serde_json::Error> {
        let msg = self.dialog.take().unwrap().resolve(data)?;
        let fun = self.fun.lock().unwrap();
        let ret = (*fun)(msg);
        Ok(ret)
    }
}

struct DialogBindingMap<'de, T: Send + 'static, U: Send + 'static, Fun: Fn(U) -> T> {
    fun: Arc<Mutex<Fun>>,
    inner: Option<Box<dyn DialogBindingTrait<'de, U>>>,
    marker: PhantomData<&'de T>,
}

impl<'de, T: Send + 'static, U: Send + 'static, Fun: Fn(U) -> T> DialogBindingTrait<'de, T> for DialogBindingMap<'de, T, U, Fun> {
    fn resolve(&mut self, data: &'de str) -> Result<T, serde_json::Error> {
        let msg = self.inner.take().unwrap().resolve(data)?;
        let fun = self.fun.lock().unwrap();
        let ret = (*fun)(msg);
        Ok(ret)
    }
}

