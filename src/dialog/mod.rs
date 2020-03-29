use std::marker::PhantomData;
use serde::Serialize;
use std::sync::{Mutex, Arc};
use serde::de::DeserializeOwned;

mod file_dialogs;
mod msg_box;

mod private {
    use crate::dialog::file_dialogs::*;
    use crate::dialog::msg_box::*;

    pub trait Sealed {}

    impl Sealed for FileOpenDialog {}
    impl Sealed for MultipleFileOpenDialog {}
    impl Sealed for FileSaveDialog {}
    impl Sealed for MessageBox {}
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

    pub(crate) fn map<U: 'static + Send, F: 'static + Send + Sync + Fn(T) -> U>(self, fun: Arc<F>) -> DialogBinding<U> {
        let inner: Box<dyn DialogBindingTrait<U>> = Box::new(DialogBindingMap{
            fun,
            inner: self.inner,
            marker: PhantomData
        });
        DialogBinding { inner: Some(inner) }
    }

    pub(crate) fn serialize(&self) -> String {
        // unwrap is fine because we only take() self.inner in resolve()
        self.inner.as_ref().unwrap().serialize()
    }
}

pub(crate) trait DialogBindingTrait<T: Send + 'static> {
    fn resolve(&mut self, data: &str) -> Result<T, serde_json::Error>;
    fn serialize(&self) -> String;
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

    fn serialize(&self) -> String {
        serde_json::to_string(&self.dialog).unwrap()
    }
}

struct DialogBindingMap<T: Send + 'static, U: Send + 'static, Fun: 'static + Send + Sync + Fn(U) -> T> {
    fun: Arc<Fun>,
    inner: Option<Box<dyn DialogBindingTrait<U>>>,
    marker: PhantomData<T>,
}

impl<T: Send + 'static, U: Send + 'static, Fun: 'static + Send + Sync + Fn(U) -> T> DialogBindingTrait<T> for DialogBindingMap<T, U, Fun> {
    fn resolve(&mut self, data: &str) -> Result<T, serde_json::Error> {
        let msg = self.inner.take().unwrap().resolve(data)?;
        let ret = (*self.fun)(msg);
        Ok(ret)
    }

    fn serialize(&self) -> String {
        // unwrap is fine because we only take() self.inner in resolve()
        self.inner.as_ref().unwrap().serialize()
    }
}
