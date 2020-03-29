use std::marker::PhantomData;
use std::sync::{Mutex, Arc};
use serde_json::{Value as JsonValue, Value};

mod private {
    use super::*;

    pub trait Sealed {}

    impl Sealed for FileOpenDialog {}
    impl Sealed for MultipleFileOpenDialog {}
    impl Sealed for MessageBox {}
    impl Sealed for FileSaveDialog {}
}

pub trait DialogSerialize {
   fn to_json(&self) -> JsonValue;
   fn from_json(value: JsonValue) -> Self;
}


pub trait Dialog: private::Sealed + DialogSerialize + std::marker::Sized {
    type Msg: DialogSerialize;

    /// Called by the runtime to produce a result based on the received
    /// data from the dialog after the user has closed it.
    fn resolve(self, data: &str) -> Result<Self::Msg, serde_json::Error> {
        todo!()
    }
}


pub enum FileOpenMsg {
    Cancel(),
    Selected(String),
}

impl DialogSerialize for FileOpenMsg {
    fn to_json(&self) -> Value {
        unimplemented!()
    }

    fn from_json(value: Value) -> Self {
        unimplemented!()
    }
}

pub struct FileOpenDialog {}

impl DialogSerialize for FileOpenDialog {
    fn to_json(&self) -> Value {
        unimplemented!()
    }

    fn from_json(value: Value) -> Self {
        unimplemented!()
    }
}

impl FileOpenDialog {
    fn new() -> Self {
        todo!()
    }

}

impl Dialog for FileOpenDialog {
    type Msg = FileOpenMsg;
}

pub enum MultipleFileOpenMsg {
    Cancel(),
    Selected(Vec<String>)
}

impl DialogSerialize for MultipleFileOpenMsg {
    fn to_json(&self) -> Value {
        unimplemented!()
    }

    fn from_json(value: Value) -> Self {
        unimplemented!()
    }
}

pub struct MultipleFileOpenDialog {}

impl DialogSerialize for MultipleFileOpenDialog {
    fn to_json(&self) -> Value {
        unimplemented!()
    }

    fn from_json(value: Value) -> Self {
        unimplemented!()
    }
}

impl Dialog for MultipleFileOpenDialog {
    type Msg = MultipleFileOpenMsg;
}

pub enum MessageBoxMsg {
    Ok(),
    Cancel(),
    Yes(),
    No(),
}

impl DialogSerialize for MessageBoxMsg {
    fn to_json(&self) -> Value {
        unimplemented!()
    }

    fn from_json(value: Value) -> Self {
        unimplemented!()
    }
}

pub struct MessageBox {

}

impl DialogSerialize for MessageBox {
    fn to_json(&self) -> Value {
        unimplemented!()
    }

    fn from_json(value: Value) -> Self {
        unimplemented!()
    }
}

impl Dialog for MessageBox {
    type Msg = MessageBoxMsg;
}

pub enum FileSaveMsg {
    SaveTo(String),
    Cancel()
}

impl DialogSerialize for FileSaveMsg {
    fn to_json(&self) -> Value {
        unimplemented!()
    }

    fn from_json(value: Value) -> Self {
        unimplemented!()
    }
}

pub struct FileSaveDialog {

}

impl Dialog for FileSaveDialog {
    type Msg = FileSaveMsg;
}

impl DialogSerialize for FileSaveDialog {
    fn to_json(&self) -> Value {
        unimplemented!()
    }

    fn from_json(value: Value) -> Self {
        unimplemented!()
    }
}

pub(crate) struct DialogBinding<T: Send + 'static> {
    inner: Option<Box<dyn DialogBindingTrait<T>>>,
}

impl<T: Send + 'static> DialogBinding<T> {
    pub(crate) fn new<D: Dialog, F: Fn(D::Msg) -> T>(&self, _dialog: D, _fun: F) -> Self {
        todo!()
    }

    pub(crate) fn resolve(mut self, data: &str) -> Result<T, serde_json::Error> {
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

