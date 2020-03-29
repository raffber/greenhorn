use serde::{Serialize, Deserialize};
use crate::dialog::Dialog;

#[derive(Serialize, Deserialize)]
pub struct MessageBox {}

#[derive(Serialize, Deserialize)]
pub enum MessageBoxMsg {
    Ok(),
    Cancel(),
    Yes(),
    No(),
}

impl Dialog for MessageBox {
    type Msg = MessageBoxMsg;
}


