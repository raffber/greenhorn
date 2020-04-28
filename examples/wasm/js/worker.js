import('../pkg').then(module => { 
    initWasmPipe(module);
    module.start();
}).catch(console.error)