"use strict";

function initWasmPipe(wasm_module) {
    window.greenhorn_push_string = (arg) => {
        window.postMessage(arg, null);
    };
    
    window.greenhorn_push_binary = (arg) => {
        window.postMessage(null, [arg.buffer]);
    };
    
    window.onmessage = (msg) => {
        let msg = msg.data;
        wasm_module.greenhorn_send_to_wasm(msg);
    };
}
