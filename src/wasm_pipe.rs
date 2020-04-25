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
use std::convert::TryInto;

pub struct WasmPipe;

fn vec_to_array(data: Vec<u8>) -> Uint8Array {
    // it would be awesome to use unsafe Uint8Array::view(). We would just need to make sure
    // we don't leak backing [u8], thus we would need to know when js is done
    // with the object, maybe after receiving the next message from js?
    let array = Uint8Array::new_with_length(data.len().try_into().unwrap());
    for k in 0..data.len() {
        array.set_index(k as u32, data[k]);
    }
    array
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
                        push_binary(vec_to_array(data));
                    },
                    rest => {
                        let msg = serde_json::to_string(&rest).unwrap();
                        push_string(msg);
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
    fn push_string(data: String);
    fn push_binary(data: Uint8Array);
}

#[wasm_bindgen]
pub fn js_to_wasm(data: String) {
    let msg: RxMsg = serde_json::from_str(&data).unwrap();
    let borrowed = PIPE.lock().unwrap();
    if let Some(pipe) = &*borrowed {
        let _ = pipe.rxmsg_tx.unbounded_send(msg);
    }
}


#[wasm_bindgen]
extern {
    fn alert(s: &str);
}


#[wasm_bindgen]
pub fn greet_two() {
    alert("greet_two!!");
}
