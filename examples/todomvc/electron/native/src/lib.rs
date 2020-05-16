use neon::prelude::*;
use ::todomvc::MainApp;
use greenhorn::{Runtime, WebSocketPipe};
use std::net::SocketAddr;
use std::str::FromStr;
use std::thread;

fn run(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let addr = SocketAddr::from_str("127.0.0.1:0").unwrap();
    let pipe = WebSocketPipe::listen_to_addr(addr);
    let port = pipe.port();
    thread::spawn(|| {
        let app = MainApp::new();
        let (rt, _control) = Runtime::new(app, pipe);
        rt.run_blocking();
    });
    Ok(cx.number(port))
}

register_module!(mut cx, {
    cx.export_function("run", run)
});
