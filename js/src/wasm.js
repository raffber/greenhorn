"use strict";

export default class Pipe {
    constructor(wasm_module) {
        this.wasm_module = wasm_module;

        window.greenhorn_push_string = (arg) => {
            let msg = JSON.parse(event.data);
            if (msg.hasOwnProperty("Patch")) {
                let data = new Uint8Array(msg.Patch);
                this.onPatch(data.buffer);
                this.sendApplied();
            } else if (msg.hasOwnProperty("Service")) {
                let service_msg = msg.Service;
                let id = service_msg[0];
                if (service_msg[1].hasOwnProperty("Frontend")) {
                    let frontend_msg = service_msg[1].Frontend;
                    this.onServiceMsg(id, frontend_msg);
                } else if (service_msg[1].hasOwnProperty("RunJs")) {
                    let run_js_msg = service_msg[1].RunJs;
                    this.onRunJsMsg(id, run_js_msg);
                } else if (service_msg[1].hasOwnProperty("LoadCss")) {
                    this.loadCss(service_msg[1].LoadCss);
                }
            } else if (msg.hasOwnProperty("LoadCss")) {
                this.loadCss(msg.LoadCss);
            } else if (msg.hasOwnProperty("RunJs")) {
                (function() {
                    eval(msg.RunJs);
                })();
            } else if (msg.hasOwnProperty("Propagate")) {
                let event = msg.Propagate.event;
                let prop = msg.Propagate.propagate;
                let default_action = msg.Propagate.default_action;
                this.onInjectEvent(event, prop, default_action);
            } else if (msg.hasOwnProperty("Dialog")) {
                this.spawnDialog(msg.Dialog);
            }
        }; 

        window.greenhorn_push_binary = (data) => {
            this.onPatch(data.buffer);
            this.sendApplied();
        }; 

        this.onPatch = (patch_data) => {};
        this.onServiceMsg = (id, service_msg) => {};
        this.onRunJsMsg = (id, run_js_msg) => {};
        this.onLoadCss = (css) => {};
        this.onInjectEvent = (event, prop, default_action) => {};
    }

    sendApplied() {
        let reply = JSON.stringify({"FrameApplied": []}); 
        this.wasm_module.js_to_wasm(reply);
    }

    spawnDialog(dialog) {
        console.log('No support for dialogs....');
        // let in_msg = { "Dialog": dialog };
        // external.invoke(JSON.stringify(in_msg));
    }

    sendEvent(id, name, evt) {
        let serialized = serializeEvent(id, name, evt);
        let msg = {
            "Event": serialized
        };
        let data = JSON.stringify(msg);
        this.wasm_module.send(data);
    }

    sendServiceMsg(id, data) {
        let msg = {
            "Service": [id, {"Frontend": data}]
        };
        let serialized = JSON.stringify(msg);
        this.wasm_module.js_to_wasm(serialized);
    }

    close() { }

    sendRpc(id, data) {
        let msg = {
            "ElementRpc": [id, data]  
        };
        let serialized = JSON.stringify(msg);
        this.wasm_module.js_to_wasm(serialized);
    }
}