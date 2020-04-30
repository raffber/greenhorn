use crate::pipe::{Pipe, RxMsg, TxMsg};
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
use futures::Sink;
use std::error::Error;
use std::pin::Pin;
use futures::task::Poll;
use std::task::Context;
use futures::StreamExt;
use std::sync::Mutex;
use lazy_static::lazy_static;
use web_sys::console::log_1;

pub struct WasmPipe;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        log_1(&format!( $( $t )* ).into());
    }
}

fn vec_to_array(data: Vec<u8>) -> Uint8Array {
    // it would be awesome to use unsafe Uint8Array::view(). We would just need to make sure
    // we don't leak backing [u8], thus we would need to know when js is done
    // with the object, maybe after receiving the next message from js?
    let data: &[u8] = &data;
    data.into()
}

impl Pipe for WasmPipe {
    type Sender = WasmSender;
    type Receiver = UnboundedReceiver<RxMsg>;

    fn split(self) -> (Self::Sender, Self::Receiver) {
        let (txmsg_tx, txmsg_rx) = unbounded();
        let (rxmsg_tx, rxmsg_rx) = unbounded();

        let endpoint = PipeJsEndpoint { rxmsg_tx };
        let mut locked = PIPE.lock().unwrap();
        assert!(!locked.is_some());
        *locked = Some(endpoint);

        crate::platform::spawn(async move {
            let mut txmsg_rx = txmsg_rx;
            while let Some(msg) = txmsg_rx.next().await {
                match msg {
                    TxMsg::Patch(data) => {
                        greenhorn_push_binary(vec_to_array(data));
                    },
                    rest => {
                        let msg = serde_json::to_string(&rest).unwrap();
                        greenhorn_push_string(msg);
                    }
                }
            }
        });
        (WasmSender(txmsg_tx), rxmsg_rx)
    }
}

impl WasmPipe {
    pub fn new() -> WasmPipe {
        WasmPipe
    }
}

struct PipeJsEndpoint {
    rxmsg_tx: UnboundedSender<RxMsg>,
}


#[derive(Clone)]
pub struct WasmSender(pub UnboundedSender<TxMsg>);

impl Sink<TxMsg> for WasmSender {
    type Error = Box<dyn Error>;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0)
            .poll_ready(cx)
            .map_err(|x| Box::new(x).into())
    }

    fn start_send(mut self: Pin<&mut Self>, item: TxMsg) -> Result<(), Self::Error> {
        Pin::new(&mut self.0)
            .start_send(item)
            .map_err(|x| Box::new(x).into())
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0)
            .poll_flush(cx)
            .map_err(|x| Box::new(x).into())
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0)
            .poll_close(cx)
            .map_err(|x| Box::new(x).into())
    }
}

lazy_static!{
    static ref PIPE: Mutex<Option<PipeJsEndpoint>> = Mutex::new(None);
}

#[wasm_bindgen]
extern "C" {
    fn greenhorn_push_string(data: String);

    fn greenhorn_push_binary(data: Uint8Array);
}

#[wasm_bindgen]
pub fn greenhorn_send_to_wasm(data: String) {
    if let Ok(msg) = serde_json::from_str::<RxMsg>(&data) {
        let borrowed = PIPE.lock().unwrap();
        if let Some(pipe) = &*borrowed {
            let _ = pipe.rxmsg_tx.unbounded_send(msg);
        }
    } else {
        log!("Garbage received: {}", data);
    }
}
