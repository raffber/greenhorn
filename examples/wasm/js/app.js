import WorkerPipe from '../../../js/src/worker_pipe.js'
import App from '../../../js/src/app.js'

onload = (evt) => {
    let pipe = new WorkerPipe("/worker.js", "/wasm_bg.wasm");
    let app = new App(pipe, document.body);
}
