var greenhorn=function(e){var t={};function i(n){if(t[n])return t[n].exports;var r=t[n]={i:n,l:!1,exports:{}};return e[n].call(r.exports,r,r.exports,i),r.l=!0,r.exports}return i.m=e,i.c=t,i.d=function(e,t,n){i.o(e,t)||Object.defineProperty(e,t,{enumerable:!0,get:n})},i.r=function(e){"undefined"!=typeof Symbol&&Symbol.toStringTag&&Object.defineProperty(e,Symbol.toStringTag,{value:"Module"}),Object.defineProperty(e,"__esModule",{value:!0})},i.t=function(e,t){if(1&t&&(e=i(e)),8&t)return e;if(4&t&&"object"==typeof e&&e&&e.__esModule)return e;var n=Object.create(null);if(i.r(n),Object.defineProperty(n,"default",{enumerable:!0,value:e}),2&t&&"string"!=typeof e)for(var r in e)i.d(n,r,function(t){return e[t]}.bind(null,r));return n},i.n=function(e){var t=e&&e.__esModule?function(){return e.default}:function(){return e};return i.d(t,"a",t),t},i.o=function(e,t){return Object.prototype.hasOwnProperty.call(e,t)},i.p="/",i(i.s=0)}([function(module,__webpack_exports__,__webpack_require__){"use strict";function _classCallCheck(e,t){if(!(e instanceof t))throw new TypeError("Cannot call a class as a function")}function _defineProperties(e,t){for(var i=0;i<t.length;i++){var n=t[i];n.enumerable=n.enumerable||!1,n.configurable=!0,"value"in n&&(n.writable=!0),Object.defineProperty(e,n.key,n)}}function _createClass(e,t,i){return t&&_defineProperties(e.prototype,t),i&&_defineProperties(e,i),e}function ownKeys(e,t){var i=Object.keys(e);if(Object.getOwnPropertySymbols){var n=Object.getOwnPropertySymbols(e);t&&(n=n.filter((function(t){return Object.getOwnPropertyDescriptor(e,t).enumerable}))),i.push.apply(i,n)}return i}function _objectSpread(e){for(var t=1;t<arguments.length;t++){var i=null!=arguments[t]?arguments[t]:{};t%2?ownKeys(Object(i),!0).forEach((function(t){_defineProperty(e,t,i[t])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(i)):ownKeys(Object(i)).forEach((function(t){Object.defineProperty(e,t,Object.getOwnPropertyDescriptor(i,t))}))}return e}function _defineProperty(e,t,i){return t in e?Object.defineProperty(e,t,{value:i,enumerable:!0,configurable:!0,writable:!0}):e[t]=i,e}__webpack_require__.r(__webpack_exports__),__webpack_require__.d(__webpack_exports__,"Pipe",(function(){return Pipe})),__webpack_require__.d(__webpack_exports__,"Application",(function(){return Application})),__webpack_require__.d(__webpack_exports__,"Patch",(function(){return Patch}));var decoder=new TextDecoder;function loadCss(e){var t=document.createElement("style");t.innerHTML=e,document.getElementsByTagName("head")[0].appendChild(t)}function serializeModifierState(e){return{alt_key:e.altKey,ctrl_key:e.ctrlKey,meta_key:e.metaKey,shift_key:e.shiftKey}}function serializePoint(e,t){return{x:e,y:t}}function serializeMouseEvent(e,t,i){return{target:{id:e},event_name:t,modifier_state:serializeModifierState(i),button:i.button,buttons:i.buttons,client:serializePoint(i.clientX,i.clientY),offset:serializePoint(i.offsetX,i.offsetY),page:serializePoint(i.pageX,i.pageY),screen:serializePoint(i.screenX,i.screenY)}}function serializeEvent(e,t,i){return i instanceof WheelEvent?{Wheel:_objectSpread({},{delta_x:i.deltaX,delta_y:i.deltaY,delta_z:i.deltaZ,delta_mode:i.deltaMode},{},serializeMouseEvent(e,t,i))}:i instanceof MouseEvent?{Mouse:serializeMouseEvent(e,t,i)}:i instanceof KeyboardEvent?{Keyboard:{target:{id:e},event_name:t,modifier_state:serializeModifierState(i),code:i.code,key:i.key,location:i.location,repeat:i.repeat,bubble:!0}}:i instanceof FocusEvent?{Focus:[{id:e},t]}:{Base:[{id:e},t]}}function deserializeEvent(e){if(e.hasOwnProperty("Keyboard")){var t=e.Keyboard,i=new KeyboardEvent(t.event_name,{code:t.code,ctrlKey:t.modifier_state.ctrl_key,key:t.key,location:t.location,altKey:t.modifier_state.alt_key,repeat:t.repeat,shiftKey:t.shift_key,metaKey:t.meta_key});return Object.defineProperty(i,"__dispatch__",{value:!0}),Object.defineProperty(i,"__id__",{value:t.target.id}),i}e.hasOwnProperty("Mouse")||e.hasOwnProperty("Wheel")||e.hasOwnProperty("Focus")||e.hasOwnProperty("Base")}function addEvent(e,t,i,n){i.addEventListener(n.name,(function(i){i.hasOwnProperty("__dispatch__")||(n.prevent_default&&i.preventDefault(),n.no_propagate&&i.stopPropagation(),e.sendEvent(t,n.name,i))}))}var Element=function(){function e(t,i){var n=arguments.length>2&&void 0!==arguments[2]?arguments[2]:[],r=arguments.length>3&&void 0!==arguments[3]?arguments[3]:[],s=arguments.length>4&&void 0!==arguments[4]?arguments[4]:[],a=arguments.length>5&&void 0!==arguments[5]?arguments[5]:null;_classCallCheck(this,e),this.id=t,this.tag=i,this.attrs=n,this.events=r,this.children=s,this.namespace=a}return _createClass(e,[{key:"create",value:function(e){if(null!==this.namespace)var t=document.createElementNS(this.namespace,this.tag);else t=document.createElement(this.tag);var i=this.id;null!==i&&t.setAttribute("__id__",i);for(var n=0;n<this.attrs.length;++n){var r=this.attrs[n];t.setAttribute(r[0],r[1])}for(n=0;n<this.events.length;++n){addEvent(e,i,t,this.events[n])}for(n=0;n<this.children.length;++n){var s=this.children[n].create(e);t.appendChild(s)}return t}}]),e}(),Text=function(){function e(t){_classCallCheck(this,e),this.text=t}return _createClass(e,[{key:"create",value:function(e){return document.createTextNode(this.text)}}]),e}(),EventHandler=function e(t,i,n){_classCallCheck(this,e),this.name=t,this.no_propagate=i,this.prevent_default=n},Context=function(){function e(t,i){_classCallCheck(this,e),this.app=i,this.id=t}return _createClass(e,[{key:"send",value:function(e){this.app.pipe.sendServiceMsg(this.id,e)}}]),e}(),Pipe=function(){function Pipe(e){_classCallCheck(this,Pipe),this.url=e,this.setupSocket(),this.onPatch=function(e){},this.onServiceMsg=function(e,t){},this.onRunJsMsg=function(e,t){}}return _createClass(Pipe,[{key:"setupSocket",value:function(){var e=this;this.connected=!1,this.socket=new WebSocket(this.url),this.socket.binaryType="arraybuffer",this.socket.onopen=function(t){e.connected=!0},this.socket.onerror=function(t){e.retryConnect()},this.socket.onclose=function(t){e.retryConnect()},this.socket.onmessage=function(t){e.onMessage(t)}}},{key:"retryConnect",value:function(){var e=this;null!=this.socket&&(this.connected=!1,this.socket=null,setTimeout((function(){e.setupSocket()}),30))}},{key:"sendApplied",value:function(){if(null!=this.socket&&this.connected){var e=JSON.stringify({FrameApplied:[]});this.socket.send(e)}}},{key:"onMessage",value:function onMessage(event){if(event.data instanceof ArrayBuffer){var data=new Uint8Array(event.data);return this.onPatch(data.buffer),void this.sendApplied()}var msg=JSON.parse(event.data);if(msg.hasOwnProperty("Patch")){var _data=new Uint8Array(msg.Patch);this.onPatch(_data.buffer),this.sendApplied()}else if(msg.hasOwnProperty("Service")){var service_msg=msg.Service,id=service_msg[0];if(service_msg[1].hasOwnProperty("Frontend")){var frontend_msg=service_msg[1].Frontend;this.onServiceMsg(id,frontend_msg)}else if(service_msg[1].hasOwnProperty("RunJs")){var run_js_msg=service_msg[1].RunJs;this.onRunJsMsg(id,run_js_msg)}else service_msg[1].hasOwnProperty("LoadCss")&&loadCss(service_msg[1].LoadCss)}else if(msg.hasOwnProperty("LoadCss"))loadCss(msg.LoadCss);else if(msg.hasOwnProperty("RunJs"))!function(){eval(msg.RunJs)}();else if(msg.hasOwnProperty("Propagate")){var _event=msg.Propagate.event,prop=msg.Propagate.propagate,default_action=msg.Propagate.default_action;this.injectEvent(_event,prop,default_action)}}},{key:"injectEvent",value:function(e,t,i){var n=deserializeEvent(e),r='[__id__="'+n.__id__+'"]';document.querySelector(r).dispatchEvent(n)}},{key:"sendEvent",value:function(e,t,i){if(null!=this.socket&&this.connected){var n={Event:serializeEvent(e,t,i)},r=JSON.stringify(n);this.socket.send(r)}}},{key:"sendServiceMsg",value:function(e,t){if(null!=this.socket&&this.connected){var i={Service:[e,{Frontend:t}]},n=JSON.stringify(i);this.socket.send(n)}}},{key:"close",value:function(){this.socket.close(),this.socket=null,this.connected=!1}}]),Pipe}(),Application=function(){function Application(e,t){_classCallCheck(this,Application),this.pipe=new Pipe(e);var i=this;if(this.root_element=t,!this.root_element.firstElementChild){var n=document.createElement("div");t.appendChild(n)}this.pipe.onPatch=function(e){i.onPatch(e)},this.pipe.onRunJsMsg=function(e,t){i.onRunJsMsg(e,t)},this.afterRender=[],this.blobs={}}return _createClass(Application,[{key:"getBlob",value:function(e){return this.blobs[e]}},{key:"registerAfterRender",value:function(e){this.afterRender.push(e)}},{key:"onRunJsMsg",value:function onRunJsMsg(id,js){var ctx=new Context(id,this);eval(js)}},{key:"onPatch",value:function(e){new Patch(e,this.root_element.firstElementChild,this).apply();var t=!0,i=!1,n=void 0;try{for(var r,s=this.afterRender[Symbol.iterator]();!(t=(r=s.next()).done);t=!0){(0,r.value)(this)}}catch(e){i=!0,n=e}finally{try{t||null==s.return||s.return()}finally{if(i)throw n}}}},{key:"close",value:function(){this.pipe.close()}},{key:"sendEvent",value:function(e,t,i){this.pipe.sendEvent(e,t,i)}}]),Application}(),Patch=function(){function e(t,i,n){_classCallCheck(this,e),this.buffer=t,this.patch=new DataView(t),this.offset=0,this.element=i,this.app=n,this.patch_funs={1:e.prototype.appendSibling,3:e.prototype.replace,4:e.prototype.changeText,5:e.prototype.ascend,6:e.prototype.descend,7:e.prototype.removeChildren,8:e.prototype.truncateSiblings,9:e.prototype.nextNode,10:e.prototype.removeAttribute,11:e.prototype.addAttribute,12:e.prototype.replaceAttribute,13:e.prototype.addBlob,14:e.prototype.removeBlob}}return _createClass(e,[{key:"popU8",value:function(){var e=this.patch.getUint8(this.offset);return this.offset+=1,e}},{key:"apply",value:function(){for(;this.offset<this.patch.byteLength;){var e=this.popU8();this.patch_funs[e].call(this)}}},{key:"deserializeNode",value:function(){var e=this.popU8();return 0===e?this.deserializeElement():1===e?this.deserializeText():void 0}},{key:"appendSibling",value:function(){var e=this.deserializeNode().create(this.app);this.element.parentNode.appendChild(e),this.element=e}},{key:"replace",value:function(){var e=this.deserializeNode().create(this.app);this.element.parentNode.replaceChild(e,this.element),this.element=e}},{key:"changeText",value:function(){var e=this.deserializeText();this.element.nodeValue=e.text}},{key:"ascend",value:function(){this.element=this.element.parentNode}},{key:"descend",value:function(){this.element=this.element.firstChild}},{key:"removeChildren",value:function(){for(;this.element.firstChild;)this.element.removeChild(this.element.firstChild)}},{key:"truncateSiblings",value:function(){for(var e=this.element.nextSibling;null!=e;){var t=e;e=e.nextSibling,this.element.parentNode.removeChild(t)}}},{key:"nextNode",value:function(){this.element=this.element.nextSibling}},{key:"removeAttribute",value:function(){var e=this.deserializeString();this.element.removeAttribute(e)}},{key:"addAttribute",value:function(){var e=this.deserializeString(),t=this.deserializeString();this.element.setAttribute(e,t)}},{key:"replaceAttribute",value:function(){var e=this.deserializeString(),t=this.deserializeString();this.element.setAttribute(e,t)}},{key:"deserializeElement",value:function(){var e=this.deserializeId(),t=this.deserializeString(),i=this.patch.getUint32(this.offset,!0);this.offset+=4;for(var n=[],r=0;r<i;++r){var s=this.deserializeString(),a=this.deserializeString();n.push([s,a])}var o=this.patch.getUint32(this.offset,!0);this.offset+=4;var l=[];for(r=0;r<o;++r){var c=this.deserializeEventHandler();l.push(c)}var u=this.patch.getUint32(this.offset,!0);this.offset+=4;var h=[];for(r=0;r<u;++r)h.push(this.deserializeNode());if(this.popU8()>0)var f=this.deserializeString();else f=null;return new Element(e,t,n,l,h,f)}},{key:"deserializeText",value:function(){var e=this.deserializeString();return new Text(e)}},{key:"deserializeOption",value:function(e){return this.popU8()>0?e():null}},{key:"deserializeId",value:function(){if(!(this.popU8()>0))return null;var e=this.patch.getUint32(this.offset,!0),t=this.patch.getUint32(this.offset+4,!0);return this.offset+=8,e+Math.pow(2,32)*t}},{key:"deserializeU64",value:function(){var e=this.patch.getUint32(this.offset,!0),t=this.patch.getUint32(this.offset+4,!0);return this.offset+=8,e+Math.pow(2,32)*t}},{key:"deserializeString",value:function(){var e=this.patch.getUint32(this.offset,!0),t=new Uint8Array(this.buffer,this.offset+4,e);return this.offset+=e+4,decoder.decode(t)}},{key:"deserializeEventHandler",value:function(){var e=this.patch.getUint8(this.offset)>0,t=this.patch.getUint8(this.offset+1)>0;this.offset+=2;var i=this.deserializeString();return new EventHandler(i,e,t)}},{key:"addBlob",value:function(){var e=this.deserializeId(),t=this.deserializeU64(),i=this.deserializeString(),n=this.patch.getUint32(this.offset,!0),r=new Uint8Array(this.buffer,this.offset+4,n),s={blob:new Blob([r],{type:i}),hash:t};this.offset+=n+4,this.app.blobs[e]=s}},{key:"removeBlob",value:function(){var e=this.deserializeId();delete this.app.blobs[e]}}]),e}()}]);