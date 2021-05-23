use std::future::Future;
use std::net::SocketAddr;
use std::str::FromStr;
use std::thread;

use tokio::net::TcpListener;
use wry::application::dpi::LogicalSize;
use wry::application::event::{Event, WindowEvent, StartCause};
use wry::application::event_loop::{ControlFlow, EventLoop};
use wry::application::window::{Window, WindowBuilder};
use wry::webview::{RpcRequest, WebViewBuilder, RpcResponse};

pub use greenhorn;
use greenhorn::dialog::native_dialogs;
use greenhorn::pipe::{RxMsg, TxMsg};
use greenhorn::WebSocketPipe;

pub struct ViewBuilder {
    pub css: Vec<String>,
    pub js: Vec<String>,
    pub port: u16,
    pub title: String,
    pub width: i32,
    pub height: i32,
}

impl<'a> ViewBuilder {
    pub fn new() -> Self {
        ViewBuilder {
            css: vec![],
            js: vec![],
            port: 44132,
            title: "".to_string(),
            width: 400,
            height: 300,
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
        // disable the context menu
        additional.push("<script>window.oncontextmenu = (e) => { e.preventDefault(); }</script>".into());
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

    pub fn run<T, Fut>(self, fun: T)
        where
            T: FnOnce(WebSocketPipe) -> Fut + Send + 'static,
            Fut: Future<Output=()> + 'static
    {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let addr = SocketAddr::from_str("127.0.0.1:0").unwrap();
        let socket = rt.block_on(TcpListener::bind(&addr)).expect("Failed to bind");
        let port = socket.local_addr().unwrap().port();

        let html = self.format_html(port);

        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(&self.title)
            .with_inner_size(LogicalSize::new(self.width, self.height))
            .build(&event_loop)
            .unwrap();

        let _webview = WebViewBuilder::new(window).unwrap()
            .with_rpc_handler(handler)
            .with_custom_protocol("greenhorn".to_string(), move |_, _| {
                return Ok(html.as_bytes().into());
            })
            .with_url("greenhorn://")
            .unwrap()
            .build()
            .unwrap();


        thread::spawn(move || {
            let fut = async {
                let pipe = WebSocketPipe::listen_to_socket(socket);
                let fut = fun(pipe);
                fut.await
            };
            rt.block_on(fut);
        });

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::NewEvents(StartCause::Init) => println!("Wry has started!"),
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => (),
            }
        });
    }
}

fn handler(_webview: &Window, request: RpcRequest) -> Option<RpcResponse> {
    let param = request.params.unwrap();
    // if this happens it's a mistake somewhere
    let rx: TxMsg = serde_json::from_value(param).expect("Invalid message received.");
    let ret = match rx {
        TxMsg::Dialog(dialog) => RxMsg::Dialog(native_dialogs::show_dialog(dialog)),
        _ => panic!()
    };
    let result = serde_json::to_value(ret).unwrap();
    Some(RpcResponse::new_result(request.id, Some(result)))
}
