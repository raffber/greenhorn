use web_view::*;
use std::thread;
use std::net::SocketAddr;
use std::str::FromStr;
use greenhorn::prelude::*;

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

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
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

    pub fn format_html(&self) -> String {
        let js_main = format!("window.onload = function() {{ app = new greenhorn.Application(\"ws://127.0.0.1:\" + {}, document.body); }}", self.port);
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

    pub fn run<T: FnOnce() -> () + Send + 'static>(self, fun: T) {
        let ret = web_view::builder()
            .title(&self.title)
            .content(Content::Html(self.format_html()))
            .size(self.width, self.height)
            .debug(self.debug)
            .resizable(true)
            .user_data(())
            .invoke_handler(|_webview, _arg| Ok(()))
            .build()
            .unwrap();

        thread::spawn(fun);

        ret.run().unwrap();
    }


}

pub fn create_app<A: App>(app: A, port: u16) -> (Runtime<A, WebsocketPipe>, RuntimeControl<A>) {
    let url  = format!("127.0.0.1:{}", port);
    let addr = SocketAddr::from_str(&url).unwrap();
    let pipe = WebsocketPipe::build(addr).listen();
    Runtime::new(app, pipe)
}
