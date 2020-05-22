"use strict";

export function setupWorker(import_path) {
    self.greenhorn_push_string = (arg) => {
        self.postMessage(arg, null);
    }

    self.greenhorn_push_binary = (arg) => {
        self.postMessage(arg.buffer, null);
    }

    self.greenhorn_set_timeout = (fun, timeout) => {
        self.setTimeout(fun, timeout);
    }
}

export function onImportWorker(wasm_module) {
    self.onmessage = (event) => {
        console.log(event);
        console.log(event.data);
        let msg = event.data;
        wasm_module.greenhorn_send_to_wasm(msg);
    }
}

