
use serde_json::Value as JsonValue;
use tinyfiledialogs::{message_box_ok, MessageBoxIcon, message_box_ok_cancel, OkCancel, message_box_yes_no, YesNo};
use tinyfiledialogs::{open_file_dialog, open_file_dialog_multi, save_file_dialog, save_file_dialog_with_filter};
use crate::dialog::{MessageBox, MsgBoxIcon, MsgBoxType, MessageBoxResult, FileSaveDialog, FileOpenDialog};
use crate::dialog::{FileSaveMsg, FileOpenMsg};


fn handle_msgbox(value: JsonValue) -> JsonValue {
    let msgbox: MessageBox = serde_json::from_value(value).unwrap();
    let icon = match msgbox.icon {
        MsgBoxIcon::Info => MessageBoxIcon::Info,
        MsgBoxIcon::Warning => MessageBoxIcon::Warning,
        MsgBoxIcon::Error => MessageBoxIcon::Error,
        MsgBoxIcon::Question => MessageBoxIcon::Question,
    };
    match msgbox.box_type {
        MsgBoxType::Ok => {
            message_box_ok(&msgbox.title, &msgbox.message, icon);
            serde_json::to_value(&MessageBoxResult::Ok).unwrap()
        },
        MsgBoxType::OkCancel => {
            let default = match msgbox.default {
                MessageBoxResult::Ok => OkCancel::Ok,
                MessageBoxResult::Cancel => OkCancel::Cancel,
                _ => panic!(),
            };
            let result = match message_box_ok_cancel(&msgbox.title, &msgbox.message, icon, default) {
                OkCancel::Cancel => MessageBoxResult::Ok,
                OkCancel::Ok => MessageBoxResult::Cancel,
            };
            serde_json::to_value(&result).unwrap()
        },
        MsgBoxType::YesNo => {
            let default = match msgbox.default {
                MessageBoxResult::Yes => YesNo::Yes,
                MessageBoxResult::No => YesNo::No,
                _ => panic!(),
            };
            let result = match message_box_yes_no(&msgbox.title, &msgbox.message, icon, default) {
                YesNo::Yes => MessageBoxResult::Yes,
                YesNo::No => MessageBoxResult::No,
            };
            serde_json::to_value(&result).unwrap()
        },
    }
}

fn handle_file_save(value: JsonValue) -> JsonValue {
    let dialog: FileSaveDialog = serde_json::from_value(value).unwrap();
    let ret = match dialog.filter {
        None => {
            match save_file_dialog(&dialog.title, &dialog.path) {
                None => FileSaveMsg::Cancel,
                Some(path) => FileSaveMsg::SaveTo(path),
            }
        },
        Some(filter) => {
            let filters: Vec<&str> = filter.filters.iter().map(|x| x.as_ref()).collect();
            let desc = &filter.description;
            match save_file_dialog_with_filter(&dialog.title, &dialog.path, &filters, desc) {
                None => FileSaveMsg::Cancel,
                Some(path) => FileSaveMsg::SaveTo(path),
            }

        },
    };
    serde_json::to_value(&ret).unwrap()
}

fn handle_file_open(value: JsonValue) -> JsonValue {
    let dialog: FileOpenDialog = serde_json::from_value(value).unwrap();
    let mut filters: Vec<&str> = Vec::new();
    let filter: Option<(&[&str], &str)> = if let Some(filter) = dialog.filter.as_ref() {
        for x in &filter.extensions {
            filters.push(x);
        }
        Some( (&filters, &filter.description) )
    } else {
        None
    };

    let ret = match dialog.multiple {
        true => {
            match open_file_dialog_multi(&dialog.title, &dialog.path, filter) {
                None => FileOpenMsg::Canceled,
                Some(files) => FileOpenMsg::SelectedMultiple(files),
            }
        },
        false => {
            match open_file_dialog(&dialog.title, &dialog.path, filter) {
                None => FileOpenMsg::Canceled,
                Some(file) => FileOpenMsg::Selected(file),
            }
        }
    };
    serde_json::to_value(&ret).unwrap()
}

/// Shows a JSON serialized dialog and returns the serialized result.
///
/// This function is blocking and needs to be called in the main thread.
pub fn show_dialog(value: JsonValue) -> JsonValue {
    let obj = value.as_object().unwrap();
    let tp = obj.get("__type__").unwrap();
    let tp_as_str = tp.as_str().unwrap();
    match tp_as_str {
        "MessageBox" => handle_msgbox(value),
        "FileSaveDialog" => handle_file_save(value),
        "FileOpenDialog" => handle_file_open(value),
        _ => panic!()
    }
}
