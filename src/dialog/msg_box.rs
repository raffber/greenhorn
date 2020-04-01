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
    pub fn new_yes_no(title: &str, message: &str) -> Self {
        Self {
            box_type: MsgBoxType::YesNo,
            title: title.to_string(),
            message: message.to_string(),
            icon: MsgBoxIcon::Question,
            default: MessageBoxResult::Yes
        }
    }

    pub fn new_ok_cancel(title: &str, message: &str) -> Self {
        Self {
            box_type: MsgBoxType::OkCancel,
            title: title.to_string(),
            message: message.to_string(),
            icon: MsgBoxIcon::Question,
            default: MessageBoxResult::Ok
        }
    }

    pub fn new_ok(title: &str, message: &str) -> Self {
        Self {
            box_type: MsgBoxType::Ok,
            title: title.to_string(),
            message: message.to_string(),
            icon: MsgBoxIcon::Info,
            default: MessageBoxResult::Ok
        }
    }

    pub fn with_icon(self, icon: MsgBoxIcon) -> Self {
        let mut x = self;
        x.icon = icon;
        x
    }

    pub fn with_default(self, default: MessageBoxResult) -> Self {
        let mut x = self;
        x.default = default;
        x
    }
}

impl Dialog for MessageBox {
    type Msg = MessageBoxResult;

    fn type_name() -> &'static str {
        "MessageBox"
    }
}

