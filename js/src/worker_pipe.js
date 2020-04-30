"use strict";

import serializeEvent from './event.js'


export default class WorkerPipe {
    constructor(worker) {
        this.worker = worker;
        let self = this;
        this.worker.onmessage = (evt) => { self.onMessage(evt) };

        this.onPatch = (patch_data) => {};
        this.onServiceMsg = (id, service_msg) => {};
        this.onRunJsMsg = (id, run_js_msg) => {};
        this.onLoadCss = (css) => {};
        this.onInjectEvent = (event, prop, default_action) => {};
    }

    onMessage(event) {
        console.log(event);
        if (event.data instanceof ArrayBuffer) {
            this.onPatch(event.data);
            this.sendApplied();
            return;
        }
        let msg = JSON.parse(event.data);
        if (msg.hasOwnProperty("Service")) {
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
    }

    sendApplied() {
        let reply = JSON.stringify({"FrameApplied": []}); 
        this.worker.postMessage(reply);
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
        this.worker.postMessage(data);
    }

    sendServiceMsg(id, data) {
        let msg = {
            "Service": [id, {"Frontend": data}]
        };
        let serialized = JSON.stringify(msg);
        this.worker.postMessage(serialized);
    }

    close() { }

    sendRpc(id, data) {
        let msg = {
            "ElementRpc": [id, data]  
        };
        let serialized = JSON.stringify(msg);
        this.worker.postMessage(serialized);
    }
}