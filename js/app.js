"use strict";

const decoder = new TextDecoder();

function loadCss(css) {
    var s = document.createElement("style");
    s.innerHTML = css;
    document.getElementsByTagName("head")[0].appendChild(s);
}

function injectEvent(event, prop, default_action) {
    // TODO: use prop, default_action
    let evt = deserializeEvent(event);
    let query = "[__id__=\"" + evt.__id__ + "\"]";
    let elem = document.querySelector(query);
    elem.dispatchEvent(evt);
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

export default class App {
    constructor(pipe, root_element, dialog_handler) {
        this.pipe = pipe;
        if (dialog_handler) {
            this.dialog_handler = dialog_handler;
        } else {
            this.dialog_handler = (app, dialog) => {};
        }
        
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
        this.pipe.onLoadCss = loadCss;
        this.pipe.onInjectEvent = injectEvent;
        this.pipe.onDialog = (dialog) => { self.onDialog(dialog); };

        this.afterRender = [];
        this.blobs = {}
    }

    onDialog(dialog) {
        this.dialog_handler(this, dialog);
    }

    getBlob(blob_id) {
        return this.blobs[blob_id];
    }

    registerAfterRender(fun) {
        this.afterRender.push(fun);
    }

    onRunJsMsg(id, js) {
        let ctx = new Context(id, this);
        (function(ctx) {
            eval(js);
        })(ctx);        
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
        this.pipe.sendEvent(id, name, evt);
    }

    send(elem, data) {
        let id = parseInt(elem.getAttribute('__id__'));
        this.pipe.sendRpc(id, data);
    }
}

export class Patch {
    constructor(patch, element, app) {
        this.buffer = patch;
        this.patch = new DataView(patch);
        this.offset = 0;
        this.element = element;
        this.app = app;
        this.current_elem_rendered = false;
        this.elements_rendered = [];
        this.blobs_changed = [];
        this.blobs_added = [];
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
            12: Patch.prototype.addAttribute,
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
        this.addToRendered();
        this.invokeRenderedEvent();

        let len = this.blobs_changed.length;
        for (var k = 0; k < len; ++k) {
            let blob = this.blobs_changed[k];
            blob.changed(blob);
        }

        len = this.blobs_added.length;
        for (var k = 0; k < len; ++k) {
            let blob = this.blobs_added[k];
            blob.added(blob);
        }

    }

    invokeRenderedEvent() {
        let len = this.elements_rendered.length;
        let evt = new Event("render");
        for (var k = 0; k < len; ++k) {
            let elem = this.elements_rendered[k];
            elem.dispatchEvent(evt);
        }
    }

    addToRendered() {
        if (this.current_elem_rendered && this.element["__has_render_event"]) {
            this.elements_rendered.push(this.element);
            this.current_elem_rendered = false;
        }
    }

    deserializeEventFunction() {
        let code = this.deserializeString();
        return new Function("event", code);
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
        let new_elem = this.deserializeNode();
        this.element.parentNode.appendChild(new_elem);
        this.element = new_elem;
    }


    replace() {
        let new_elem = this.deserializeNode();
        this.element.parentNode.replaceChild(new_elem, this.element);
        this.element = new_elem;
    }

    changeText() {
        let text = this.deserializeText();
        this.element.parentNode.replaceChild(text, this.element);
        this.element = text;
    }

    ascend() {
        this.addToRendered();
        this.element = this.element.parentNode;
    }

    descend() {
        this.addToRendered();
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
        this.addToRendered();
        let len = this.patch.getUint32(this.offset, true);
        this.offset += 4;
        for (let k = 0; k < len; ++k) {
            this.element = this.element.nextSibling;
        }
    }

    removeAttribute() {
        let attr = this.deserializeString();
        this.element.removeAttribute(attr);
        this.current_elem_rendered = true;
    }

    addAttribute() {
        let key = this.deserializeString();
        let value = this.deserializeString();
        this.updateAttribute(this.element, key, value);
        this.current_elem_rendered = true;
    }

    removeJsEvent() {
        let attr = this.deserializeString();
        let attr_key = '__' + attr;
        let attr_value = this.element[attr_key];
        this.element.removeEventListener(attr, attr_value);
        this.element[attr_key] = undefined;
        this.current_elem_rendered = true;
    }

    addJsEvent() {
        let key = this.deserializeString();
        let fun = this.deserializeEventFunction();
        if (key == "render") {
            this.element["__has_render_event"] = true;
        }
        this.element['__' + key] = fun;
        this.element.addEventListener(key, fun);
        this.current_elem_rendered = true;
    }

    replaceJsEvent() {
        let key = this.deserializeString();
        let fun = this.deserializeEventFunction();
        let key_attr = '__' + key;
        let attr_value = this.element[key_attr];
        this.element.removeEventListener(key, attr_value);
        this.element[key_attr] = fun;
        this.element.addEventListener(key, fun);
        this.current_elem_rendered = true;
    }

    addChildren() {
        let len = this.patch.getUint32(this.offset, true);
        this.offset += 4;
        for (var k = 0; k < len; ++k) {
            let elem = this.deserializeNode();
            this.element.appendChild(elem);
        }
    }

    updateAttribute(elem, key, value) {
        if (key == "checked" && elem instanceof HTMLInputElement) {
            elem.checked = (value == 'true');
        } else {
            elem.setAttribute(key, value);
        }
    }

    deserializeElement() {
        let tag = this.deserializeString();
        
        let hasNamespace = this.popU8() > 0;
        if (hasNamespace) {
            var elem = document.createElementNS(this.deserializeString(), tag);
        } else {
            var elem = document.createElement(tag);
        }

        let id = this.deserializeId();
        if (id !== null) {
            elem.setAttribute("__id__", id);
        }

        // attributes
        let attr_len = this.patch.getUint32(this.offset, true);
        this.offset += 4;
        for (var k = 0; k < attr_len; ++k) {
            let key = this.deserializeString();
            let value = this.deserializeString();
            this.updateAttribute(elem, key, value);
        }

        // event listeners
        let events_len = this.patch.getUint32(this.offset, true);
        this.offset += 4;
        for (var k = 0; k < events_len; ++k) {
            let evt = this.deserializeEventHandler();
            addEvent(this.app, id, elem, evt);
        }

        // js events
        let js_events_len = this.patch.getUint32(this.offset, true);
        this.offset += 4;
        let push_to_rendered = false;
        for (var k = 0; k < js_events_len; ++k) {
            let key = this.deserializeString();
            if (key == "render") {
                elem["__has_render_event"] = true;
                push_to_rendered = true;
            }
            let fun = this.deserializeEventFunction();
            elem['__' + key] = fun;
            elem.addEventListener(key, fun);
        }

        // children
        let children_len = this.patch.getUint32(this.offset, true);
        this.offset += 4;
        for (var k = 0; k < children_len; ++k) {
            elem.appendChild(this.deserializeNode());
        }

        if (push_to_rendered) {
            this.elements_rendered.push(elem);
        }        

        return elem;
    }

    deserializeText() {
        let text = this.deserializeString();
        return document.createTextNode(text); 
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
        this.offset += len + 4;
        let blob = {'blob': new Blob([view], {"type": mime_type}), 'hash': hash, 'changed': null, 'added': null};

        let changed = this.app.blobs.hasOwnProperty(id);

        let add_available = this.popU8() > 0;
        if (add_available) {
            let code = this.deserializeString();
            let fun = new Function("blob", code);
            blob.added = fun;
            if (!changed) {
                this.blobs_added.push(blob);
            }            
        }

        let changed_available = this.popU8() > 0;
        if (changed_available) {
            let code = this.deserializeString();
            let fun = new Function("blob", code);
            blob.changed = fun;
            if (changed) {
                this.blobs_changed.push(blob);
            }            
        }
        this.app.blobs[id] = blob;
    }

    removeBlob() {
        let id = this.deserializeId();
        delete this.app.blobs[id];
    }
}
