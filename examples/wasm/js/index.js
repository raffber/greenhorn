
import('../pkg').then(module => {
    window.push_string = (arg) => {
        console.log("push_string")
        // module.js_to_wasm('{ "Dialog": [] }');
        console.log(arg)
    };


    window.push_binary = (arg) => {
        console.log("push_binary")
        console.log(arg)
    };


    module.start();
}).catch(console.error)
