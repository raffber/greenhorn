"use strict";

window.greenhorn_push_string = (arg) => {
    postMessage(arg, null);
};

window.greenhorn_push_binary = (arg) => {
    postMessage(null, [arg.buffer]);
}

var wasm_module = null;

onmessage = (msg) => {
    let msg = msg.data;
    if (msg.hasOwnProperty("LoadWasm")) {
        WebAssembly.instantiateStreaming(fetch(msg.LoadWasm)).then( (m) => {
            wasm_module = m;
        });
    } else if (wasm_module !== null) {
        wasm_module.greenhorn_send_to_wasm(msg);
    }
};

