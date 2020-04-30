
self.greenhorn_push_string = function(arg) {
    console.log("push_string", arg);
    self.postMessage(arg, null);
}

self.greenhorn_push_binary = function(arg) {
    self.postMessage(arg.buffer, null);
    console.log("push_binary", arg);
}

import('../pkg/wasm.js').then(wasm_module => { 
    self.onmessage = (event) => {
        console.log(event);
        console.log(event.data);
        let msg = event.data;
        wasm_module.greenhorn_send_to_wasm(msg);
    }
}).catch(console.error)
