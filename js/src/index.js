"use strict";

var msgpack = require("msgpack-lite");

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

function serializeMouseEvent(evt) {
    return {
        "modifier_state": serializeModifierState(evt),
        "button": evt.button,
        "buttons": evt.buttons,
        "client": serializePoint(evt.clientX, evt.clientY),
        "movement": serializePoint(evt.movementX, evt.movementY),
        "offset": serializePoint(evt.offsetX, evt.offsetY),
        "page": serializePoint(evt.pageX, evt.pageY),
        "screen": serializePoint(evt.screenX, evt.screenY)
    };
}

function serializeEvent(id, name, evt) {

    if (evt instanceof WheelEvent) {
        return {
            "Wheel": [
                {"id": id}, name,
                {
                    "mouse_event": serializeMouseEvent(evt),
                    "delta_x": evt.deltaX,
                    "delta_y": evt.deltaY,
                    "delta_z": evt.deltaZ,
                    "delta_mode": evt.deltaMode
                }
            ]
        }
    } else if (evt instanceof MouseEvent) {
        return {
            "Mouse": [
                {"id": id}, name,
                serializeMouseEvent(evt)
            ]
        }
    } else if (evt instanceof KeyboardEvent) {
        return {
            "Keyboard": [
                {"id": id}, name,
                {
                    "modifier_state": serializeModifierState(evt),
                    "code": evt.code,
                    "key": evt.key,
                    "location": evt.location,
                    "repeat": evt.repeat
                }
            ]
        }
    } else if (evt instanceof FocusEvent) {
        return {
            "Focus": [{"id": id}, name]
        }
    } else {
        return {
            "Base": [{"id": id}, name]
        }
    }
}

function addEvent(app, id, elem, evt) {
    // TODO: also support once
    // TODO: also support useCapture
    // TODO: let passive = !evt.prevent_default;
    elem.addEventListener(evt.name, function(e) {
        if (evt.prevent_default) {
            e.preventDefault();
        }
        if (evt.no_propagate) { 
            e.stopPropagation();
        }
        app.sendEvent(id, evt.name, e);
    });
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
        this.socket = new WebSocket(url);
        this.socket.binaryType = "arraybuffer";
        let self = this;
        this.socket.onopen = (e) => {};
        this.socket.onerror = (e) => {};
        this.socket.onclose = (e) => {};

        this.socket.onmessage = (e) => {
            self.onMessage(e);
        };


        this.onPatch = (patch_data) => {};
        this.onServiceMsg = (id, service_msg) => {};
        this.onRunJsMsg = (id, run_js_msg) => {};
    }

    onMessage(event) {
        let msg = JSON.parse(event.data);
        if (msg.hasOwnProperty("Patch")) {
            let data = new Uint8Array(msg.Patch);
            this.onPatch(data.buffer);
            let reply = JSON.stringify({
                "FrameApplied": []
            });
            this.socket.send(reply);
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
            eval(msg.RunJs);
        }
    }

    sendEvent(id, name, evt) {
        let serialized = serializeEvent(id, name, evt);
        let msg = {
            "Event": serialized
        };
        let data = JSON.stringify(msg);
        this.socket.send(data);
    }

    sendServiceMsg(id, data) {
        let msg = {
            "Service": [id, {"Frontend": data}]
        };
        let serialized = JSON.stringify(msg);
        this.socket.send(serialized);
    }

    close() {
        this.socket.close();
    }
}

export class Application {
    constructor(url, root_element) {
        this.pipe = new Pipe(url);
        let self = this;
        let elem = document.createElement("div");
        root_element.appendChild(elem);
        this.root_element = root_element;

        this.pipe.onPatch = (e) => {
            self.onPatch(e);
        }

        this.pipe.onRunJsMsg = (id, js) => {
            self.onRunJsMsg(id, js);
        }
    }

    onRunJsMsg(id, js) {
        let ctx = new Context(id, this);
        eval(js);
    }

    onPatch(patch_data) {
        let patch = new Patch(patch_data, this.root_element.firstChild, this);
        patch.apply();
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
            1: Patch.prototype.appendChild,
            3: Patch.prototype.replace,
            4: Patch.prototype.changeText,
            5: Patch.prototype.ascend,
            6: Patch.prototype.descend,
            7: Patch.prototype.removeChildren,
            8: Patch.prototype.truncateChildren,
            9: Patch.prototype.nextNode,
            10: Patch.prototype.removeAttribute,
            11: Patch.prototype.addAttribute,
            12: Patch.prototype.replaceAttribute,
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

    appendChild() {
        console.log("appendChild");
        let node = this.deserializeNode();
        this.element.appendChild(node.create(this.app));
    }


    replace() {
        let node = this.deserializeNode();
        this.element.parentNode.replaceChild(node.create(this.app), this.element);
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
        console.log("removeChildren")
        while (this.element.firstChild) {
            this.element.removeChild(this.element.firstChild);
          }
    }

    truncateChildren() {
        console.log("truncateChildren")
        let next = this.element.nextSibling;
        while (next != null) {
            let to_remove = next;
            next = next.nextSibling;
            this.element.removeChild(to_remove);
        }
    }

    nextNode() {
        this.element = this.element.nextSibling;
    }

    removeAttribute() {
        console.log("removeAttribute")
        let attr = this.deserializeString();
        this.element.removeAttribute(attr);
    }

    addAttribute() {
        console.log("addAttribute")
        let key = this.deserializeString();
        let value = this.deserializeString();
        this.element.setAttribute(key, value);
    }

    replaceAttribute() {
        let key = this.deserializeString();
        let value = this.deserializeString();
        this.element.setAttribute(key, value);
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
        console.log("deserializeOption");
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

    
}
