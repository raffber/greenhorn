"use strict";

const decoder = new TextDecoder();

function loadCss(css) {
    var s = document.createElement("style");
    s.innerHTML = css;
    document.getElementsByTagName("head")[0].appendChild(s);
}

function serializeModifierState(evt) {
    return {
        "alt_key": evt.altKey,
        "ctrl_key": evt.ctrlKey,
        "meta_key": evt.metaKey,
        "shift_key": evt.shiftKey
    };
}

function serializePoint(x,y) {
    return {
        "x": x,
        "y": y
    };
}

function serializeMouseEvent(id, name, evt) {
    return {
        "target": {"id": id},
        "event_name": name,
        "modifier_state": serializeModifierState(evt),
        "button": evt.button,
        "buttons": evt.buttons,
        "client": serializePoint(evt.clientX, evt.clientY),
        "offset": serializePoint(evt.offsetX, evt.offsetY),
        "page": serializePoint(evt.pageX, evt.pageY),
        "screen": serializePoint(evt.screenX, evt.screenY)
    };
}

function serializeTargetValue(target) {
    let v =  target.value;
    if (typeof v === "string") {
        return {"Text": v};
    } else if (typeof v === "boolean") {
        return {"Bool": v};
    } else if (typeof v === "number") {
        return {"Number": v};
    } else {
        return "NoValue";
    }    
}

function serializeEvent(id, name, evt) {
    if (evt instanceof WheelEvent) {
        let wheel =  {
            "delta_x": evt.deltaX,
            "delta_y": evt.deltaY,
            "delta_z": evt.deltaZ,
            "delta_mode": evt.deltaMode
        };
        return {
            "Wheel": { ...wheel, ...serializeMouseEvent(id, name, evt) }
        }
    } else if (evt instanceof MouseEvent) {
        return {
            "Mouse": serializeMouseEvent(id, name, evt)
        }
    } else if (evt instanceof KeyboardEvent) {
        return {
            "Keyboard":
                {
                    "target": {"id": id},
                    "event_name": name,            
                    "modifier_state": serializeModifierState(evt),
                    "code": evt.code,
                    "key": evt.key,
                    "location": evt.location,
                    "repeat": evt.repeat,
                    "bubble": true,
                }
        }
    } else if (evt instanceof FocusEvent) {
        return {
            "Focus": [{"id": id}, name]
        }
    } else if (evt instanceof ChangeEvent) {
        return {
            "Change": {
                "target": {"id": id},
                event_name: name,
                value: serializeTargetValue(evt)
            }
        }
    } else {
        return {
            "Base": [{"id": id}, name]
        }
    }
}

function deserializeEvent(event) {
    if (event.hasOwnProperty("Keyboard")) {
        let evt = event.Keyboard;
        let ret = new KeyboardEvent(evt.event_name, {
            "code": evt.code,
            "ctrlKey": evt.modifier_state.ctrl_key,
            "key": evt.key,
            "location": evt.location,
            "altKey": evt.modifier_state.alt_key,
            "repeat": evt.repeat,
            "shiftKey": evt.shift_key,
            "metaKey": evt.meta_key,
        });
        Object.defineProperty(ret, "__dispatch__", {value: true});
        Object.defineProperty(ret, "__id__", {value: evt.target.id});
        return ret;
    } else if (event.hasOwnProperty("Mouse")) {
        // TODO: 
    } else if (event.hasOwnProperty("Wheel")) {
        // TODO: 
    } else if (event.hasOwnProperty("Focus")) {
        // TODO: 
    } else if (event.hasOwnProperty("Base")) {
        // TODO: 
    }
}

function addEvent(app, id, elem, evt) {
    // TODO: also support once
    // TODO: also support useCapture
    elem.addEventListener(evt.name, function(e) {
        if (e.hasOwnProperty("__dispatch__")) {
            return;
        }
        if (evt.prevent_default) {
            e.preventDefault();
        }
        if (evt.no_propagate) { 
            e.stopPropagation();
        }
        app.sendEvent(id, evt.name, e);
    }, {'passive': !evt.prevent_default});
}

class Element {
    constructor(id, tag, attrs=[], events=[], children=[], namespace=null) {
        this.id = id;
        this.tag = tag;
        this.attrs = attrs;
        this.events = events;
        this.children = children;
        this.namespace = namespace;
    }

    create(app) {
        if (this.namespace !== null) {
            var elem = document.createElementNS(this.namespace, this.tag);
        } else {
            var elem = document.createElement(this.tag);
        }
        
        let id = this.id;
        if (id !== null) {
            elem.setAttribute("__id__", id);
        }
        for (var k = 0; k < this.attrs.length; ++k) {
            let attr = this.attrs[k];
            elem.setAttribute(attr[0], attr[1]);
        }
        for (var k = 0; k < this.events.length; ++k) {
            let evt = this.events[k];
            addEvent(app, id, elem, evt);
        }
        for (var k = 0; k < this.children.length; ++k) {
            let child = this.children[k].create(app);
            elem.appendChild(child);
        }
        return elem;
    }
}

class Text {
    constructor(text) {
        this.text = text;
    }

    create(app) {
        return document.createTextNode(this.text);
    }
}

class EventHandler {
    constructor(name, no_propagate, prevent_default) {
        this.name = name;
        this.no_propagate = no_propagate;
        this.prevent_default = prevent_default;
    }
}

class Context {
    constructor(id, app) {
        this.app = app;
        this.id = id;
    }

    send(data) {
        this.app.pipe.sendServiceMsg(this.id, data);
    }
}


export class Pipe {
    constructor(url) {
        this.url = url;
        this.setupSocket();
        this.onPatch = (patch_data) => {};
        this.onServiceMsg = (id, service_msg) => {};
        this.onRunJsMsg = (id, run_js_msg) => {};
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
                loadCss(service_msg[1].LoadCss);
            }
        } else if (msg.hasOwnProperty("LoadCss")) {
            loadCss(msg.LoadCss);
        } else if (msg.hasOwnProperty("RunJs")) {
            (function() {
                eval(msg.RunJs);
            })();
        } else if (msg.hasOwnProperty("Propagate")) {
            let event = msg.Propagate.event;
            let prop = msg.Propagate.propagate;
            let default_action = msg.Propagate.default_action;
            this.injectEvent(event, prop, default_action);
        } else if (msg.hasOwnProperty("Dialog")) {
            this.spawnDialog(msg.Dialog);
        }
    }

    spawnDialog(dialog) {
        let in_msg = {
            "Dialog": dialog
        };
        external.invoke(JSON.stringify(in_msg));
    }

    injectEvent(event, prop, default_action) {
        // TODO: use prop, default_action
        let evt = deserializeEvent(event);
        let query = "[__id__=\"" + evt.__id__ + "\"]";
        let elem = document.querySelector(query);
        elem.dispatchEvent(evt);
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
}

export class Application {
    constructor(url, root_element) {
        this.pipe = new Pipe(url);
        let self = this;
        this.root_element = root_element;
        if (!this.root_element.firstElementChild) {
            let elem = document.createElement("div");
            root_element.appendChild(elem);
        }

        this.pipe.onPatch = (e) => {
            self.onPatch(e);
        }

        this.pipe.onRunJsMsg = (id, js) => {
            self.onRunJsMsg(id, js);
        }
        this.afterRender = [];
        this.blobs = {}
    }

    getBlob(blob_id) {
        return this.blobs[blob_id];
    }

    registerAfterRender(fun) {
        this.afterRender.push(fun);
    }

    onRunJsMsg(id, js) {
        let ctx = new Context(id, this);
        eval(js);
    }

    sendReturnMessage(ret_msg) {
        let data = JSON.stringify(ret_msg);
        this.pipe.socket.send(data);
    }

    onPatch(patch_data) {
        let patch = new Patch(patch_data, this.root_element.firstElementChild, this);
        let self = this;
        window.requestAnimationFrame(() => {
            patch.apply();
            for (const cb of self.afterRender) {
                cb(self);
            }
        });
    }

    close() {
        this.pipe.close();
    }

    sendEvent(id, name, evt) {
        this.pipe.sendEvent(id, name, evt)
    }
}

export class Patch {
    constructor(patch, element, app) {
        this.buffer = patch;
        this.patch = new DataView(patch);
        this.offset = 0;
        this.element = element;
        this.app = app;
        this.patch_funs = {
            1: Patch.prototype.appendSibling,
            3: Patch.prototype.replace,
            4: Patch.prototype.changeText,
            5: Patch.prototype.ascend,
            6: Patch.prototype.descend,
            7: Patch.prototype.removeChildren,
            8: Patch.prototype.truncateSiblings,
            9: Patch.prototype.nextNode,
            10: Patch.prototype.removeAttribute,
            11: Patch.prototype.addAttribute,
            12: Patch.prototype.replaceAttribute,
            13: Patch.prototype.addBlob,
            14: Patch.prototype.removeBlob,
            15: Patch.prototype.removeJsEvent,
            16: Patch.prototype.addJsEvent,
            17: Patch.prototype.replaceJsEvent,
            18: Patch.prototype.addChildren,
        }
    }

    popU8() {
        let ret = this.patch.getUint8(this.offset);
        this.offset += 1;
        return ret;
    }

    apply() {
        while (this.offset < this.patch.byteLength) {
            let x = this.popU8();
            let fun = this.patch_funs[x];
            fun.call(this);
        }
    }

    deserializeNode() {
        let x = this.popU8();
        if (x === 0) {
            return this.deserializeElement();
        } else if (x === 1) {
            return this.deserializeText();
        }
    }

    appendSibling() {
        let node = this.deserializeNode();
        let new_elem = node.create(this.app);
        this.element.parentNode.appendChild(new_elem);
        this.element = new_elem;
    }


    replace() {
        let node = this.deserializeNode();
        let new_elem = node.create(this.app);
        this.element.parentNode.replaceChild(new_elem, this.element);
        this.element = new_elem;
    }

    changeText() {
        let text = this.deserializeText();
        this.element.nodeValue = text.text;
    }

    ascend() {
        this.element = this.element.parentNode;
    }

    descend() {
        this.element = this.element.firstChild;
    }

    removeChildren() {
        while (this.element.firstChild) {
            this.element.removeChild(this.element.firstChild);
          }
    }

    truncateSiblings() {
        let next = this.element.nextSibling;
        while (next != null) {
            let to_remove = next;
            next = next.nextSibling;
            this.element.parentNode.removeChild(to_remove);
        }
    }

    nextNode() {
        let len = this.patch.getUint32(this.offset, true);
        this.offset += 4;
        for (let k = 0; k < len; ++k) {
            this.element = this.element.nextSibling;
        }
    }

    removeAttribute() {
        let attr = this.deserializeString();
        this.element.removeAttribute(attr);
    }

    addAttribute() {
        let key = this.deserializeString();
        let value = this.deserializeString();
        this.element.setAttribute(key, value);
    }

    replaceAttribute() {
        let key = this.deserializeString();
        let value = this.deserializeString();
        this.element.setAttribute(key, value);
    }

    removeJsEvent() {
        let attr = this.deserializeString();
        let attr_key = '__' + attr;
        let attr_value = this.element[attr_key];
        this.element.removeEventListener(attr_value);
        this.element[attr_key] = undefined;
    }

    addJsEvent() {
        let key = this.deserializeString();
        let value = this.deserializeString();
        let app = this.app;
        let fun = function() {
            eval(value)
        }();
        this.element['__' + key] = fun;
        this.element.addEventListener(key, fun);
    }

    replaceJsEvent() {
        let key = this.deserializeString();
        let value = this.deserializeString();
        let app = this.app;
        let fun = function() {
            eval(value)
        }();
        let key_attr = '__' + key;
        let attr_value = this.element[key_attr];
        this.element.removeEventListener(attr_value);
        this.element[key_attr] = fun;
        this.element.addEventListener(key, fun);
    }

    addChildren() {
        let len = this.patch.getUint32(this.offset, true);
        this.offset += 4;
        for (var k = 0; k < len; ++k) {
            let node = this.deserializeNode();
            this.element.appendChild(node.create());
        }
    }

    deserializeElement() {
        let id = this.deserializeId();
        let tag = this.deserializeString();
        let attr_len = this.patch.getUint32(this.offset, true);
        this.offset += 4;
        let attrs = [];
        for (var k = 0; k < attr_len; ++k) {
            let key = this.deserializeString();
            let value = this.deserializeString();
            attrs.push([key, value]);
        }
        let events_len = this.patch.getUint32(this.offset, true);
        this.offset += 4;
        let events = [];
        for (var k = 0; k < events_len; ++k) {
            let event = this.deserializeEventHandler();
            events.push(event);
        }
        let children_len = this.patch.getUint32(this.offset, true);
        this.offset += 4;
        let children = [];
        for (var k = 0; k < children_len; ++k) {
            children.push(this.deserializeNode());
        }
        let self = this;
        let hasNamespace = this.popU8() > 0;
        if (hasNamespace) {
            var namespace = this.deserializeString();
        } else {
            var namespace = null;
        }
        return new Element(id, tag, attrs, events, children, namespace);
    }

    deserializeText() {
        let text = this.deserializeString();
        return new Text(text); 
    }

    deserializeOption(deserializer) {
        let available = this.popU8() > 0;
        if (available) {
            return deserializer();
        }
        return null;
    }

    deserializeId() {
        let available = this.popU8() > 0;
        if (!available) {
            return null;
        }
        let lo = this.patch.getUint32(this.offset, true);
        let hi = this.patch.getUint32(this.offset + 4, true);
        this.offset += 8;
        return lo + (2**32)*hi;
    }

    deserializeU64() {
        let lo = this.patch.getUint32(this.offset, true);
        let hi = this.patch.getUint32(this.offset + 4, true);
        this.offset += 8;
        return lo + (2**32)*hi;
    }

    deserializeString() {
        let len = this.patch.getUint32(this.offset, true);
        let view = new Uint8Array(this.buffer, this.offset + 4, len);
        this.offset += len + 4;
        return decoder.decode(view);
    }

    deserializeEventHandler() {
        let no_prop = this.patch.getUint8(this.offset) > 0;
        let prevent_default = this.patch.getUint8(this.offset + 1) > 0;
        this.offset += 2;
        let name = this.deserializeString();
        return new EventHandler(name, no_prop, prevent_default);
    }

    addBlob() {
        let id = this.deserializeId();
        let hash = this.deserializeU64();
        let mime_type = this.deserializeString();
        let len = this.patch.getUint32(this.offset, true);
        let view = new Uint8Array(this.buffer, this.offset + 4, len);
        let blob = {'blob': new Blob([view], {"type": mime_type}), 'hash': hash};
        this.offset += len + 4;
        this.app.blobs[id] = blob;
    }

    removeBlob() {
        let id = this.deserializeId();
        delete this.app.blobs[id];
    }
}
