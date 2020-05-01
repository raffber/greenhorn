import { setupWorker, onImportWorker } from '../../../../js/src/wasm.js'

setupWorker();

import('../pkg/wasm.js').then(wasm_module => {
    onImportWorker(wasm_module)
}).catch(console.error)

