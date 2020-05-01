
self.greenhorn_push_string = (arg) => {
    console.log("push_string", arg);
    self.postMessage(arg, null);
}

self.greenhorn_push_binary = (arg) => {
    self.postMessage(arg.buffer, null);
    console.log("push_binary", arg);
}

self.greenhorn_set_timeout = (fun, timeout) => {
    self.setTimeout(fun, timeout);
}

import('../pkg/wasm.js').then(wasm_module => { 
    self.onmessage = (event) => {
        console.log(event);
        console.log(event.data);
        let msg = event.data;
        wasm_module.greenhorn_send_to_wasm(msg);
    }
}).catch(console.error)
