use web_view::*;
use std::thread;
use std::net::SocketAddr;
use std::str::FromStr;
use greenhorn::prelude::*;
use async_std::net::TcpListener;
use async_std::task;
use greenhorn::dialog::{MessageBox, MsgBoxIcon, MsgBoxType, MessageBoxResult, FileSaveDialog, FileOpenDialog};
use greenhorn::dialog::{FileSaveMsg, FileOpenMsg};
use serde_json::Value as JsonValue;
use tinyfiledialogs::{message_box_ok, MessageBoxIcon, message_box_ok_cancel, OkCancel, message_box_yes_no, YesNo};
use tinyfiledialogs::{open_file_dialog, open_file_dialog_multi, save_file_dialog, save_file_dialog_with_filter};
use greenhorn::pipe::{RxMsg, TxMsg};

pub struct ViewBuilder {
    pub css: Vec<String>,
    pub js: Vec<String>,
    pub port: u16,
    pub title: String,
    pub width: i32,
    pub height: i32,
    pub debug: bool,
}

impl<'a> ViewBuilder {
    pub fn new() -> Self {
        #[cfg(debug_assertions)]
        let debug = true;
        #[cfg(not(debug_assertions))]
        let debug = false;
        ViewBuilder {
            css: vec![],
            js: vec![],
            port: 44132,
            title: "".to_string(),
            width: 400,
            height: 300,
            debug
        }
    }

    pub fn css<T: Into<String>>(mut self, css: T) -> Self {
        self.css.push(css.into());
        self
    }

    pub fn js<T: Into<String>>(mut self, js: T) -> Self {
        self.js.push(js.into());
        self
    }

    pub fn title<T: Into<String>>(mut self, title: T) -> Self {
        self.title = title.into();
        self
    }

    pub fn size(mut self, width: i32, height: i32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn format_html(&self, port: u16) -> String {
        let js_main = format!("window.onload = function() {{ app = new greenhorn.Application(\"ws://127.0.0.1:\" + {}, document.body); }}", port);
        let js_lib = include_str!("../res/bundle.js");
        let mut additional = Vec::new();
        for x in &self.js {
            additional.push(format!("<script>{}</script>", x));
        }
        for x in &self.css {
            additional.push(format!("<style>{}</style>", x));
        }
        if !self.debug {
            // disable the context menu
            additional.push("<script>window.oncontextmenu = (e) => { e.preventDefault(); }</script>".into());
        }
        let additional = additional.join("\n");
        let html_content = format!("<!DOCTYPE html>
        <html>
            <head> <meta charset=\"UTF-8\">
                <script>{}</script>
                <script>{}</script>
                {}
            </head>
            <body></body>
        </html>", js_lib, js_main, additional);
        html_content
    }

    pub fn run<T: FnOnce(WebsocketPipe) -> () + Send + 'static>(self, fun: T) {
        let addr = SocketAddr::from_str("127.0.0.1:0").unwrap();

        let socket = task::block_on(async {
            TcpListener::bind(&addr).await
        }).expect("Failed to bind");

        let port = socket.local_addr().unwrap().port();

        let ret = web_view::builder()
            .title(&self.title)
            .content(Content::Html(self.format_html(port)))
            .size(self.width, self.height)
            .debug(self.debug)
            .resizable(true)
            .user_data(())
            .invoke_handler(|webview, arg| {
                handler(webview, arg);
                Ok(())
            })
            .build()
            .unwrap();


        let pipe = WebsocketPipe::listen_to_socket(socket);

        let thread = thread::spawn(move || fun(pipe) );

        ret.run().unwrap();

        thread.join().unwrap();
    }
}

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
            JsonValue::Null
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
        for x in &filter.filters {
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

fn handle_dialog(_webview: &mut WebView<()>, value: JsonValue) -> TxMsg {
    let obj = value.as_object().unwrap();
    let tp = obj.get("__type__").unwrap();
    let tp_as_str = tp.as_str().unwrap();
    let ret = match tp_as_str {
        "MessageBox" => handle_msgbox(value),
        "FileSaveDialog" => handle_file_save(value),
        "FileOpenDialog" => handle_file_open(value),
        _ => panic!()
    };
    TxMsg::Dialog(ret)
}

fn handler(webview: &mut WebView<()>, arg: &str) {
     // if this happens it's a mistake somewhere
    let rx: RxMsg = serde_json::from_str(arg).expect("Invalid message received.");
    let ret = match rx {
        RxMsg::Dialog(dialog) => handle_dialog(webview, dialog),
        _ => panic!()
    };
    // this already produces an escaped js string
    let ret = serde_json::to_string(&ret).unwrap();
    let arg = format!("window.app.sendReturnMessage({});", ret);
    webview.eval(&arg).unwrap();
}
