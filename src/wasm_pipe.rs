use crate::pipe::{Pipe, RxMsg, TxMsg};
use crate::platform::spawn;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use js_sys::Uint8Array;

struct WasmPipe {}

impl Pipe for WasmPipe {
    type Sender = UnboundedSender<TxMsg>;
    type Receiver = UnboundedReceiver<RxMsg>;

    fn split(self) -> (Self::Sender, Self::Receiver) {
        let (txmsg_tx, txmsg_rx) = unbounded();
        let (rxmsg_tx, rxmsg_rx) = unbounded();
        spawn(async move {});
        (txmsg_tx, rxmsg_rx)
    }
}

#[wasm_bindgen(module = "/wasm-pipe.js")]
extern "C" {
    fn push_string(data: String);
    fn push_binary(data: Uint8Array);
}

#[wasm_bindgen]
fn js_to_wasm(data: String) {}
