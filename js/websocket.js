"use strict";

import serializeEvent from './event.js'


export default class Pipe {
    constructor(url) {
        this.url = url;
        this.setupSocket();
        this.onPatch = (patch_data) => {};
        this.onServiceMsg = (id, service_msg) => {};
        this.onRunJsMsg = (id, run_js_msg) => {};
        this.onLoadCss = (css) => {};
        this.onInjectEvent = (event, prop, default_action) => {};
        this.onDialog = (dialog) => {}
    }

    setupSocket() {
        let self = this;
        this.connected = false;
        this.socket = new WebSocket(this.url);
        this.socket.binaryType = "arraybuffer";
        this.socket.onopen = (e) => { 
            self.connected = true;
        };
        this.socket.onerror = (e) => {
            self.retryConnect();
        }
        this.socket.onclose = (e) => { 
            self.retryConnect();
        };
        this.socket.onmessage = (e) => { self.onMessage(e); };
    }

    retryConnect() {
        let self = this;
        
        if (this.socket == null) {
            return;
        }
        this.connected = false;
        this.socket = null;
        setTimeout(() => {
            self.setupSocket();
        }, 30);
    }

    sendApplied() {
        if (this.socket == null || !this.connected) {
            return;
        }
        let reply = JSON.stringify({"FrameApplied": []}); 
        this.socket.send(reply);
    }

    onMessage(event) {
        // conclusion on performance testing:
        // JSON.parse is much faster then msgpack.decode()
        // json serialization on server is approx 2x slower
        // however, since serialization may be run in parallel
        // on server but must be run on a single thread here
        // we are better off just using json

        // in case we get binary data it must be a Patch
        if (event.data instanceof ArrayBuffer) {
            let data = new Uint8Array(event.data);
            this.onPatch(data.buffer);
            this.sendApplied();
            return;
        }

        // in case we get text data in can be any type of message
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
                this.onLoadCss(service_msg[1].LoadCss);
            }
        } else if (msg.hasOwnProperty("LoadCss")) {
            this.onloadCss(msg.LoadCss);
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
            this.onDialog(msg.Dialog);
        }
    }

    sendEvent(id, name, evt) {
        if (this.socket == null || !this.connected) {
            return;
        }
        let serialized = serializeEvent(id, name, evt);
        let msg = {
            "Event": serialized
        };
        let data = JSON.stringify(msg);
        this.socket.send(data);
    }

    sendServiceMsg(id, data) {
        if (this.socket == null || !this.connected) {
            return;
        }
        let msg = {
            "Service": [id, {"Frontend": data}]
        };
        let serialized = JSON.stringify(msg);
        this.socket.send(serialized);
    }

    close() {
        this.socket.close();
        this.socket = null;
        this.connected = false;
    }

    sendRpc(id, data) {
        let msg = {
            "ElementRpc": [id, data]  
        };
        let serialized = JSON.stringify(msg);
        this.socket.send(serialized);
    }
}