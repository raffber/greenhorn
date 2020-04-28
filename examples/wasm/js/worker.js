import initWasmPipe from '../../../js/src/wasm.js'

import('../pkg').then(module => { 
    initWasmPipe(module);
}).catch(console.error)