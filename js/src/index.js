"use strict";

const CBOR = require('cbor');

const decoder = new TextDecoder();

class Element {
    constructor(id, tag, attrs=[], events=[], children=[]) {
        this.id = id;
        this.tag = tag;
        this.attrs = attrs;
        this.events = events;
        this.children = children;
    }

    create(app) {
        let elem = document.createElement(this.tag);
        elem.setAttribute("__id__", this.id.toString())
        for (var k = 0; k < this.attrs.length; ++k) {
            let attr = this.attrs[k];
            elem.setAttribute(attr[0], attr[1]);
        }
        for (var k = 0; k < this.events.length; ++k) {
            let evt = this.events[k];
            elem.addEventListener(evt.name, function(e) {
                app.sendEvent(this.id, e);
            })
        }
        for (var k = 0; k < this.children.length; ++k) {
            let child = this.children[k].create(app);
            elem.appendChild(child);
        }
        return elem;
    }
}

class Text {
    constructor(id, text) {
        this.id = id;
        this.text = text;
    }

    create(app) {
        return document.createTextNode(this.text);
    }
}

class Id {
    constructor(lo,hi) {
        this.lo = lo;
        this.hi = hi;
    }

    equals(other) {
        return this.lo == other.lo && this.hi == other.hi;
    }

    toString() {
        return this.hi.toString(16) + "-" + this.lo.toString(16);
    }
}

class EventHandler {
    constructor(name, no_propagate, prevent_default) {
        this.name = name;
        this.no_propagate = no_propagate;
        this.prevent_default = prevent_default;
    }
}

function id_from_string(str) {
    let splits = str.split("-");
    return new Id(
        parseInt(splits[1], 16),
        parseInt(splits[0], 16),
    );
}

function id_from_dataview(data_view, offset) {
    let lo = data_view.getUint32(offset, true);
    let hi = data_view.getUint32(offset + 4, true);
    return new Id(lo,hi);
}

export class Pipe {
    constructor(url) {
        this.socket = new WebSocket(url);
        this.socket.binaryType = "arraybuffer";
        let self = this;
        this.socket.onopen = (e) => {
            self.onOpen(e);
        };

        this.socket.onmessage = (e) => {
            self.onMessage(e);
        };

        this.socket.onerror = (e) => {
            self.onError(e);
        };

        this.socket.onclose = (e) => {
            self.onClose(e);
        };

        this.onPatch = (patch_data) => {};
    }

    onOpen(event) {
        console.log("onOpen");
    }

    onMessage(event) {
        // let data = new Uint8Array(event.data);
        // console.log(data);
        // let msg = CBOR.decode(event.data);
        let msg = JSON.parse(event.data);
        let data = new Uint8Array(msg.Patch);
        this.onPatch(data.buffer);
        
        console.log("onMessage");
    }

    onClose(event) {
        console.log("onClose");
    }

    onError(event) {
        console.log("onError");
    }

    sendEvent(evt, id) {
        console.log("sendEvent");
    }

    close() {
        this.socket.close();
    }
}

export class Application {
    constructor(url, root_element) {
        this.pipe = new Pipe(url);
        let self = this;
        this.pipe.onPatch = (e) => {
            self.onPatch(e);
        }
        this.root_element = root_element;
    }

    onPatch(patch_data) {
        let patch = new Patch(patch_data, this.root_element, this);
        patch.apply();
    }

    close() {
        this.pipe.close();
    }

    sendEvent(id, evt) {
        this.pipe.sendEvent(id, evt)
    }
}

export class Patch {
    constructor(patch, element, app) {
        this.buffer = patch;
        this.patch = new DataView(patch);
        this.offset = 0;
        this.element = element;
        this.app = app;
        this.child_idx = [0];
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
            13: Patch.prototype.removeEvent,
            14: Patch.prototype.addEvent,
            15: Patch.prototype.changeNamespace,
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
        console.log("deserializeNode");
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
        console.log("replace");
        let node = this.deserializeNode();
        this.element.parentNode.replaceChild(node.create(this.app), this.element);
    }

    changeText() {
        console.log("changeText")
        let text = this.deserializeText();
        this.element.innerText = text.text;
    }

    ascend() {
        console.log("ascend")
        this.element = this.element.parentNode;
    }

    descend() {
        console.log("descend")
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
        // TODO: ....
    }

    nextNode() {
        console.log("nextNode");
        let idx = this.element.parent.childNodes.indexOf(this.element);
        this.element = this.element.childNodes[idx+1];
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
        console.log("replaceAttribute")
        let key = this.deserializeString();
        let value = this.deserializeString();
        this.element.setAttribute(key, value);
    }

    removeEvent() {
        console.log("removeEvent")
        let name = this.deserializeString();
        // TODO: ...
    }

    addEvent() {
        console.log("addEvent")
        let evt = this.deserializeEventHandler();
        // TODO: ...
    }

    changeNamespace() {
        console.log("changeNamespace")
        let ns = this.deserializeOption(this.deserializeString);
        // TOOD: ...
    }

    deserializeElement() {
        console.log("deserializeElement");
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
        return new Element(id, tag, attrs, events, children);
    }

    deserializeText() {
        console.log("deserializeText");
        let id = this.deserializeId();
        let text = this.deserializeString();
        return new Text(id, text); 
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
        console.log("deserializeId");
        let ret = id_from_dataview(this.patch, this.offset);
        this.offset += 8;
        return ret;
    }

    deserializeString() {
        console.log("deserializeString");
        let len = this.patch.getUint32(this.offset, true);
        let view = new Uint8Array(this.buffer, this.offset + 4, len);
        this.offset += len + 4;
        return decoder.decode(view);
    }

    deserializeEventHandler() {
        console.log("deserializeEventHandler");
        let no_prop = this.patch.getUint8(this.offset) > 0;
        let prevent_default = this.patch.getUint8(this.offset + 1) > 0;
        this.offset += 2;
        let name = this.deserializeString();
        return new EventHandler(name, no_prop, prevent_default);
    }

    
}
