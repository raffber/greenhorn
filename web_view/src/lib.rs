use web_view::*;
use std::thread;
use std::net::SocketAddr;
use std::str::FromStr;
use greenhorn::prelude::*;
use async_std::net::TcpListener;
use async_std::task;
use greenhorn::pipe::{TxMsg, RxMsg};
use greenhorn::dialog::native_dialogs;

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
        let js_main = format!("window.onload = function() {{ \
            let pipe = new greenhorn.Pipe(\"ws://127.0.0.1:\" + {});
            let dialog_handler = (app, dialog) => {{
                let in_msg = {{ 'Dialog': dialog }};
                external.invoke(JSON.stringify(in_msg));
            }};
            let app = new greenhorn.App(pipe, document.body, dialog_handler);
            window.app = app;
        }}", port);
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

    pub fn run<T: FnOnce(WebSocketPipe) -> () + Send + 'static>(self, fun: T) {
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


        let pipe = WebSocketPipe::listen_to_socket(socket);

        let thread = thread::spawn(move || fun(pipe) );

        ret.run().unwrap();

        thread.join().unwrap();
    }
}

fn handler(webview: &mut WebView<()>, arg: &str) {
     // if this happens it's a mistake somewhere
    let rx: TxMsg = serde_json::from_str(arg).expect("Invalid message received.");
    let ret = match rx {
        TxMsg::Dialog(dialog) => RxMsg::Dialog(native_dialogs::show_dialog( dialog)),
        _ => panic!()
    };
    // this already produces an escaped js string
    let ret = serde_json::to_string(&ret).unwrap();
    let arg = format!("window.app.sendReturnMessage({});", ret);
    println!("{:?}", arg);
    webview.eval(&arg).unwrap();
}
