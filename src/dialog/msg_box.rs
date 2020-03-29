use serde::{Serialize, Deserialize};
use crate::dialog::Dialog;

#[derive(Clone, Serialize, Deserialize)]
pub enum MsgBoxType {
    Ok,
    OkCancel,
    YesNo,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum MsgBoxIcon {
    Info,
    Warning,
    Error,
    Question
}

#[derive(Clone, Serialize, Deserialize)]
pub enum MessageBoxMsg {
    Ok(),
    Cancel(),
    Yes(),
    No(),
}

#[derive(Serialize, Deserialize)]
pub struct MessageBox {
    box_type: MsgBoxType,
    title: String,
    message: String,
    icon: MsgBoxIcon,
}

impl MessageBox {
    pub fn new(box_type: MsgBoxType, title: &str, message: &str, icon: MsgBoxIcon) -> Self {
        Self {
            box_type,
            title: title.to_string(),
            message: message.to_string(),
            icon
        }
    }
}

impl Dialog for MessageBox {
    type Msg = MessageBoxMsg;

    fn type_name() -> &'static str {
        "MessageBox"
    }
}

