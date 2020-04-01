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
pub enum MessageBoxResult {
    Ok,
    Cancel,
    Yes,
    No,
}

#[derive(Serialize, Deserialize)]
pub struct MessageBox {
    pub box_type: MsgBoxType,
    pub title: String,
    pub message: String,
    pub icon: MsgBoxIcon,
    pub default: MessageBoxResult,
}

impl MessageBox {
    pub fn new(box_type: MsgBoxType, title: &str, message: &str, icon: MsgBoxIcon) -> Self {
        Self {
            box_type,
            title: title.to_string(),
            message: message.to_string(),
            icon,
            default: MessageBoxResult::Ok,
        }
    }
}

impl Dialog for MessageBox {
    type Msg = MessageBoxResult;

    fn type_name() -> &'static str {
        "MessageBox"
    }
}

