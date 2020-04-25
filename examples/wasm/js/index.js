import Pipe from '../../../js/src/wasm.js'
import App from '../../../js/src/app.js'

import('../pkg').then(module => { 
    let pipe = new Pipe(module);
    let app = new App(pipe, document.body);
    module.start();
}).catch(console.error)
