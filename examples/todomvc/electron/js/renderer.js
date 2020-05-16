
var remote = require('electron').remote;
import '../../lib/dist/styles.css';
import Application from '../../../../js/src/app.js';
import Pipe from '../../../../js/src/websocket.js';

let app = null;

window.onload = function() {
    const port = remote.getGlobal('port');
    let pipe = new Pipe("ws://127.0.0.1:" + port);
    app = new Application(pipe, document.body);
}

window.onunload = function() {
    app.pipe.close();
}