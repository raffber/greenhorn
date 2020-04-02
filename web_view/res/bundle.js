var greenhorn=function(e){var t={};function n(i){if(t[i])return t[i].exports;var r=t[i]={i:i,l:!1,exports:{}};return e[i].call(r.exports,r,r.exports,n),r.l=!0,r.exports}return n.m=e,n.c=t,n.d=function(e,t,i){n.o(e,t)||Object.defineProperty(e,t,{enumerable:!0,get:i})},n.r=function(e){"undefined"!=typeof Symbol&&Symbol.toStringTag&&Object.defineProperty(e,Symbol.toStringTag,{value:"Module"}),Object.defineProperty(e,"__esModule",{value:!0})},n.t=function(e,t){if(1&t&&(e=n(e)),8&t)return e;if(4&t&&"object"==typeof e&&e&&e.__esModule)return e;var i=Object.create(null);if(n.r(i),Object.defineProperty(i,"default",{enumerable:!0,value:e}),2&t&&"string"!=typeof e)for(var r in e)n.d(i,r,function(t){return e[t]}.bind(null,r));return i},n.n=function(e){var t=e&&e.__esModule?function(){return e.default}:function(){return e};return n.d(t,"a",t),t},n.o=function(e,t){return Object.prototype.hasOwnProperty.call(e,t)},n.p="/",n(n.s=0)}([function(module,__webpack_exports__,__webpack_require__){"use strict";function _createForOfIteratorHelper(e){if("undefined"==typeof Symbol||null==e[Symbol.iterator]){if(Array.isArray(e)||(e=_unsupportedIterableToArray(e))){var t=0,n=function(){};return{s:n,n:function(){return t>=e.length?{done:!0}:{done:!1,value:e[t++]}},e:function(e){throw e},f:n}}throw new TypeError("Invalid attempt to iterate non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method.")}var i,r,a=!0,s=!1;return{s:function(){i=e[Symbol.iterator]()},n:function(){var e=i.next();return a=e.done,e},e:function(e){s=!0,r=e},f:function(){try{a||null==i.return||i.return()}finally{if(s)throw r}}}}function _unsupportedIterableToArray(e,t){if(e){if("string"==typeof e)return _arrayLikeToArray(e,t);var n=Object.prototype.toString.call(e).slice(8,-1);return"Object"===n&&e.constructor&&(n=e.constructor.name),"Map"===n||"Set"===n?Array.from(n):"Arguments"===n||/^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)?_arrayLikeToArray(e,t):void 0}}function _arrayLikeToArray(e,t){(null==t||t>e.length)&&(t=e.length);for(var n=0,i=new Array(t);n<t;n++)i[n]=e[n];return i}function _classCallCheck(e,t){if(!(e instanceof t))throw new TypeError("Cannot call a class as a function")}function _defineProperties(e,t){for(var n=0;n<t.length;n++){var i=t[n];i.enumerable=i.enumerable||!1,i.configurable=!0,"value"in i&&(i.writable=!0),Object.defineProperty(e,i.key,i)}}function _createClass(e,t,n){return t&&_defineProperties(e.prototype,t),n&&_defineProperties(e,n),e}function ownKeys(e,t){var n=Object.keys(e);if(Object.getOwnPropertySymbols){var i=Object.getOwnPropertySymbols(e);t&&(i=i.filter((function(t){return Object.getOwnPropertyDescriptor(e,t).enumerable}))),n.push.apply(n,i)}return n}function _objectSpread(e){for(var t=1;t<arguments.length;t++){var n=null!=arguments[t]?arguments[t]:{};t%2?ownKeys(Object(n),!0).forEach((function(t){_defineProperty(e,t,n[t])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(n)):ownKeys(Object(n)).forEach((function(t){Object.defineProperty(e,t,Object.getOwnPropertyDescriptor(n,t))}))}return e}function _defineProperty(e,t,n){return t in e?Object.defineProperty(e,t,{value:n,enumerable:!0,configurable:!0,writable:!0}):e[t]=n,e}__webpack_require__.r(__webpack_exports__),__webpack_require__.d(__webpack_exports__,"Pipe",(function(){return Pipe})),__webpack_require__.d(__webpack_exports__,"Application",(function(){return Application})),__webpack_require__.d(__webpack_exports__,"Patch",(function(){return Patch}));var decoder=new TextDecoder;function loadCss(e){var t=document.createElement("style");t.innerHTML=e,document.getElementsByTagName("head")[0].appendChild(t)}function serializeModifierState(e){return{alt_key:e.altKey,ctrl_key:e.ctrlKey,meta_key:e.metaKey,shift_key:e.shiftKey}}function serializePoint(e,t){return{x:e,y:t}}function serializeMouseEvent(e,t,n){return{target:{id:e},event_name:t,modifier_state:serializeModifierState(n),button:n.button,buttons:n.buttons,client:serializePoint(n.clientX,n.clientY),offset:serializePoint(n.offsetX,n.offsetY),page:serializePoint(n.pageX,n.pageY),screen:serializePoint(n.screenX,n.screenY)}}function serializeTargetValue(e){var t=e.value;return"string"==typeof t?{Text:t}:"boolean"==typeof t?{Bool:t}:"number"==typeof t?{Number:t}:"NoValue"}function serializeEvent(e,t,n){return n instanceof WheelEvent?{Wheel:_objectSpread({},{delta_x:n.deltaX,delta_y:n.deltaY,delta_z:n.deltaZ,delta_mode:n.deltaMode},{},serializeMouseEvent(e,t,n))}:n instanceof MouseEvent?{Mouse:serializeMouseEvent(e,t,n)}:n instanceof KeyboardEvent?{Keyboard:{target:{id:e},event_name:t,modifier_state:serializeModifierState(n),code:n.code,key:n.key,location:n.location,repeat:n.repeat,bubble:!0}}:n instanceof FocusEvent?{Focus:[{id:e},t]}:n instanceof ChangeEvent?{Change:{target:{id:e},event_name:t,value:serializeTargetValue(n)}}:{Base:[{id:e},t]}}function deserializeEvent(e){if(e.hasOwnProperty("Keyboard")){var t=e.Keyboard,n=new KeyboardEvent(t.event_name,{code:t.code,ctrlKey:t.modifier_state.ctrl_key,key:t.key,location:t.location,altKey:t.modifier_state.alt_key,repeat:t.repeat,shiftKey:t.shift_key,metaKey:t.meta_key});return Object.defineProperty(n,"__dispatch__",{value:!0}),Object.defineProperty(n,"__id__",{value:t.target.id}),n}e.hasOwnProperty("Mouse")||e.hasOwnProperty("Wheel")||e.hasOwnProperty("Focus")||e.hasOwnProperty("Base")}function addEvent(e,t,n,i){n.addEventListener(i.name,(function(n){n.hasOwnProperty("__dispatch__")||(i.prevent_default&&n.preventDefault(),i.no_propagate&&n.stopPropagation(),e.sendEvent(t,i.name,n))}),{passive:!i.prevent_default})}var Element=function(){function e(t,n){var i=arguments.length>2&&void 0!==arguments[2]?arguments[2]:[],r=arguments.length>3&&void 0!==arguments[3]?arguments[3]:[],a=arguments.length>4&&void 0!==arguments[4]?arguments[4]:[],s=arguments.length>5&&void 0!==arguments[5]?arguments[5]:null;_classCallCheck(this,e),this.id=t,this.tag=n,this.attrs=i,this.events=r,this.children=a,this.namespace=s}return _createClass(e,[{key:"create",value:function(e){if(null!==this.namespace)var t=document.createElementNS(this.namespace,this.tag);else t=document.createElement(this.tag);var n=this.id;null!==n&&t.setAttribute("__id__",n);for(var i=0;i<this.attrs.length;++i){var r=this.attrs[i];t.setAttribute(r[0],r[1])}for(i=0;i<this.events.length;++i){addEvent(e,n,t,this.events[i])}for(i=0;i<this.children.length;++i){var a=this.children[i].create(e);t.appendChild(a)}return t}}]),e}(),Text=function(){function e(t){_classCallCheck(this,e),this.text=t}return _createClass(e,[{key:"create",value:function(e){return document.createTextNode(this.text)}}]),e}(),EventHandler=function e(t,n,i){_classCallCheck(this,e),this.name=t,this.no_propagate=n,this.prevent_default=i},Context=function(){function e(t,n){_classCallCheck(this,e),this.app=n,this.id=t}return _createClass(e,[{key:"send",value:function(e){this.app.pipe.sendServiceMsg(this.id,e)}}]),e}(),Pipe=function(){function Pipe(e){_classCallCheck(this,Pipe),this.url=e,this.setupSocket(),this.onPatch=function(e){},this.onServiceMsg=function(e,t){},this.onRunJsMsg=function(e,t){}}return _createClass(Pipe,[{key:"setupSocket",value:function(){var e=this;this.connected=!1,this.socket=new WebSocket(this.url),this.socket.binaryType="arraybuffer",this.socket.onopen=function(t){e.connected=!0},this.socket.onerror=function(t){e.retryConnect()},this.socket.onclose=function(t){e.retryConnect()},this.socket.onmessage=function(t){e.onMessage(t)}}},{key:"retryConnect",value:function(){var e=this;null!=this.socket&&(this.connected=!1,this.socket=null,setTimeout((function(){e.setupSocket()}),30))}},{key:"sendApplied",value:function(){if(null!=this.socket&&this.connected){var e=JSON.stringify({FrameApplied:[]});this.socket.send(e)}}},{key:"onMessage",value:function onMessage(event){if(event.data instanceof ArrayBuffer){var data=new Uint8Array(event.data);return this.onPatch(data.buffer),void this.sendApplied()}var msg=JSON.parse(event.data);if(msg.hasOwnProperty("Patch")){var _data=new Uint8Array(msg.Patch);this.onPatch(_data.buffer),this.sendApplied()}else if(msg.hasOwnProperty("Service")){var service_msg=msg.Service,id=service_msg[0];if(service_msg[1].hasOwnProperty("Frontend")){var frontend_msg=service_msg[1].Frontend;this.onServiceMsg(id,frontend_msg)}else if(service_msg[1].hasOwnProperty("RunJs")){var run_js_msg=service_msg[1].RunJs;this.onRunJsMsg(id,run_js_msg)}else service_msg[1].hasOwnProperty("LoadCss")&&loadCss(service_msg[1].LoadCss)}else if(msg.hasOwnProperty("LoadCss"))loadCss(msg.LoadCss);else if(msg.hasOwnProperty("RunJs"))!function(){eval(msg.RunJs)}();else if(msg.hasOwnProperty("Propagate")){var _event=msg.Propagate.event,prop=msg.Propagate.propagate,default_action=msg.Propagate.default_action;this.injectEvent(_event,prop,default_action)}else msg.hasOwnProperty("Dialog")&&this.spawnDialog(msg.Dialog)}},{key:"spawnDialog",value:function(e){var t={Dialog:e};external.invoke(JSON.stringify(t))}},{key:"injectEvent",value:function(e,t,n){var i=deserializeEvent(e),r='[__id__="'+i.__id__+'"]';document.querySelector(r).dispatchEvent(i)}},{key:"sendEvent",value:function(e,t,n){if(null!=this.socket&&this.connected){var i={Event:serializeEvent(e,t,n)},r=JSON.stringify(i);this.socket.send(r)}}},{key:"sendServiceMsg",value:function(e,t){if(null!=this.socket&&this.connected){var n={Service:[e,{Frontend:t}]},i=JSON.stringify(n);this.socket.send(i)}}},{key:"close",value:function(){this.socket.close(),this.socket=null,this.connected=!1}}]),Pipe}(),Application=function(){function Application(e,t){_classCallCheck(this,Application),this.pipe=new Pipe(e);var n=this;if(this.root_element=t,!this.root_element.firstElementChild){var i=document.createElement("div");t.appendChild(i)}this.pipe.onPatch=function(e){n.onPatch(e)},this.pipe.onRunJsMsg=function(e,t){n.onRunJsMsg(e,t)},this.afterRender=[],this.blobs={}}return _createClass(Application,[{key:"getBlob",value:function(e){return this.blobs[e]}},{key:"registerAfterRender",value:function(e){this.afterRender.push(e)}},{key:"onRunJsMsg",value:function onRunJsMsg(id,js){var ctx=new Context(id,this);eval(js)}},{key:"sendReturnMessage",value:function(e){var t=JSON.stringify(e);this.pipe.socket.send(t)}},{key:"onPatch",value:function(e){var t=new Patch(e,this.root_element.firstElementChild,this),n=this;window.requestAnimationFrame((function(){t.apply();var e,i=_createForOfIteratorHelper(n.afterRender);try{for(i.s();!(e=i.n()).done;){(0,e.value)(n)}}catch(e){i.e(e)}finally{i.f()}}))}},{key:"close",value:function(){this.pipe.close()}},{key:"sendEvent",value:function(e,t,n){this.pipe.sendEvent(e,t,n)}}]),Application}(),Patch=function(){function Patch(e,t,n){_classCallCheck(this,Patch),this.buffer=e,this.patch=new DataView(e),this.offset=0,this.element=t,this.app=n,this.patch_funs={1:Patch.prototype.appendSibling,3:Patch.prototype.replace,4:Patch.prototype.changeText,5:Patch.prototype.ascend,6:Patch.prototype.descend,7:Patch.prototype.removeChildren,8:Patch.prototype.truncateSiblings,9:Patch.prototype.nextNode,10:Patch.prototype.removeAttribute,11:Patch.prototype.addAttribute,12:Patch.prototype.replaceAttribute,13:Patch.prototype.addBlob,14:Patch.prototype.removeBlob,15:Patch.prototype.removeJsEvent,16:Patch.prototype.addJsEvent,17:Patch.prototype.replaceJsEvent,18:Patch.prototype.addChildren}}return _createClass(Patch,[{key:"popU8",value:function(){var e=this.patch.getUint8(this.offset);return this.offset+=1,e}},{key:"apply",value:function(){for(;this.offset<this.patch.byteLength;){var e=this.popU8();this.patch_funs[e].call(this)}}},{key:"deserializeNode",value:function(){var e=this.popU8();return 0===e?this.deserializeElement():1===e?this.deserializeText():void 0}},{key:"appendSibling",value:function(){var e=this.deserializeNode().create(this.app);this.element.parentNode.appendChild(e),this.element=e}},{key:"replace",value:function(){var e=this.deserializeNode().create(this.app);this.element.parentNode.replaceChild(e,this.element),this.element=e}},{key:"changeText",value:function(){var e=this.deserializeText();this.element.nodeValue=e.text}},{key:"ascend",value:function(){this.element=this.element.parentNode}},{key:"descend",value:function(){this.element=this.element.firstChild}},{key:"removeChildren",value:function(){for(;this.element.firstChild;)this.element.removeChild(this.element.firstChild)}},{key:"truncateSiblings",value:function(){for(var e=this.element.nextSibling;null!=e;){var t=e;e=e.nextSibling,this.element.parentNode.removeChild(t)}}},{key:"nextNode",value:function(){var e=this.patch.getUint32(this.offset,!0);this.offset+=4;for(var t=0;t<e;++t)this.element=this.element.nextSibling}},{key:"removeAttribute",value:function(){var e=this.deserializeString();this.element.removeAttribute(e)}},{key:"addAttribute",value:function(){var e=this.deserializeString(),t=this.deserializeString();this.element.setAttribute(e,t)}},{key:"replaceAttribute",value:function(){var e=this.deserializeString(),t=this.deserializeString();this.element.setAttribute(e,t)}},{key:"removeJsEvent",value:function(){var e="__"+this.deserializeString(),t=this.element[e];this.element.removeEventListener(t),this.element[e]=void 0}},{key:"addJsEvent",value:function addJsEvent(){var key=this.deserializeString(),value=this.deserializeString(),app=this.app,fun=function(){eval(value)}();this.element["__"+key]=fun,this.element.addEventListener(key,fun)}},{key:"replaceJsEvent",value:function replaceJsEvent(){var key=this.deserializeString(),value=this.deserializeString(),app=this.app,fun=function(){eval(value)}(),key_attr="__"+key,attr_value=this.element[key_attr];this.element.removeEventListener(attr_value),this.element[key_attr]=fun,this.element.addEventListener(key,fun)}},{key:"addChildren",value:function(){var e=this.patch.getUint32(this.offset,!0);this.offset+=4;for(var t=0;t<e;++t){var n=this.deserializeNode();this.element.appendChild(n.create())}}},{key:"deserializeElement",value:function(){var e=this.deserializeId(),t=this.deserializeString(),n=this.patch.getUint32(this.offset,!0);this.offset+=4;for(var i=[],r=0;r<n;++r){var a=this.deserializeString(),s=this.deserializeString();i.push([a,s])}var o=this.patch.getUint32(this.offset,!0);this.offset+=4;var l=[];for(r=0;r<o;++r){var c=this.deserializeEventHandler();l.push(c)}var u=this.patch.getUint32(this.offset,!0);this.offset+=4;var h=[];for(r=0;r<u;++r)h.push(this.deserializeNode());if(this.popU8()>0)var f=this.deserializeString();else f=null;return new Element(e,t,i,l,h,f)}},{key:"deserializeText",value:function(){var e=this.deserializeString();return new Text(e)}},{key:"deserializeOption",value:function(e){return this.popU8()>0?e():null}},{key:"deserializeId",value:function(){if(!(this.popU8()>0))return null;var e=this.patch.getUint32(this.offset,!0),t=this.patch.getUint32(this.offset+4,!0);return this.offset+=8,e+Math.pow(2,32)*t}},{key:"deserializeU64",value:function(){var e=this.patch.getUint32(this.offset,!0),t=this.patch.getUint32(this.offset+4,!0);return this.offset+=8,e+Math.pow(2,32)*t}},{key:"deserializeString",value:function(){var e=this.patch.getUint32(this.offset,!0),t=new Uint8Array(this.buffer,this.offset+4,e);return this.offset+=e+4,decoder.decode(t)}},{key:"deserializeEventHandler",value:function(){var e=this.patch.getUint8(this.offset)>0,t=this.patch.getUint8(this.offset+1)>0;this.offset+=2;var n=this.deserializeString();return new EventHandler(n,e,t)}},{key:"addBlob",value:function(){var e=this.deserializeId(),t=this.deserializeU64(),n=this.deserializeString(),i=this.patch.getUint32(this.offset,!0),r=new Uint8Array(this.buffer,this.offset+4,i),a={blob:new Blob([r],{type:n}),hash:t};this.offset+=i+4,this.app.blobs[e]=a}},{key:"removeBlob",value:function(){var e=this.deserializeId();delete this.app.blobs[e]}}]),Patch}()}]);