//! This modules implements data structures for handling pop-up message boxes.

use crate::dialog::Dialog;
use serde::{Deserialize, Serialize};

/// Defines the type of message box according to the buttons on the window
#[derive(Clone, Serialize, Deserialize)]
pub enum MsgBoxType {
    Ok,
    OkCancel,
    YesNo,
}

/// Defines the message box icon
#[derive(Clone, Serialize, Deserialize)]
pub enum MsgBoxIcon {
    Info,
    Warning,
    Error,
    Question,
}

/// The result of the message box the dialog resolves to once the
/// dialog has been closed.
#[derive(Clone, Serialize, Deserialize)]
pub enum MessageBoxResult {
    Ok,
    Cancel,
    Yes,
    No,
}

/// Represents the message box dialog.
///
/// # Example
///
/// ```
/// # use greenhorn::context::Context;
/// # use greenhorn::dialog::{MessageBox, MessageBoxResult};
/// #
/// enum Msg {
///     MsgBox(MessageBoxResult),
///     ShowMsgBox,
/// }
///
/// fn update(msg: Msg, ctx: Context<Msg>) {
///     match msg {
///         Msg::MsgBox(msg) => {
///             match msg {
///                 MessageBoxResult::Ok => {println!("Ok button was pressed")}
///                 MessageBoxResult::Cancel => {println!("Cancel button was pressed")}
///                 _ => {}
///             }
///         }
///
///         Msg::ShowMsgBox => {
///             let dialog = MessageBox::new_ok_cancel("Window title", "Hello, World!");
///             ctx.dialog(dialog, Msg::MsgBox);
///         }
///     }
/// }
/// ```
///
#[derive(Serialize, Deserialize)]
pub struct MessageBox {
    pub box_type: MsgBoxType,
    pub title: String,
    pub message: String,
    pub icon: MsgBoxIcon,
    pub default: MessageBoxResult,
}

impl MessageBox {
    /// Create a new dialog with a "Yes" and "No" button
    pub fn new_yes_no(title: &str, message: &str) -> Self {
        Self {
            box_type: MsgBoxType::YesNo,
            title: title.to_string(),
            message: message.to_string(),
            icon: MsgBoxIcon::Question,
            default: MessageBoxResult::Yes,
        }
    }

    /// Create a new dialog with an "Ok" and "Cancel" button
    pub fn new_ok_cancel(title: &str, message: &str) -> Self {
        Self {
            box_type: MsgBoxType::OkCancel,
            title: title.to_string(),
            message: message.to_string(),
            icon: MsgBoxIcon::Question,
            default: MessageBoxResult::Ok,
        }
    }

    /// Create a new dialog with an "Ok" button
    pub fn new_ok(title: &str, message: &str) -> Self {
        Self {
            box_type: MsgBoxType::Ok,
            title: title.to_string(),
            message: message.to_string(),
            icon: MsgBoxIcon::Info,
            default: MessageBoxResult::Ok,
        }
    }

    /// Customize the icon of the dialog
    pub fn with_icon(self, icon: MsgBoxIcon) -> Self {
        let mut x = self;
        x.icon = icon;
        x
    }

    /// Setup default result of the dialog
    pub fn with_default_result(self, default: MessageBoxResult) -> Self {
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
