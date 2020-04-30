import WorkerPipe from '../../../js/src/worker_pipe.js'
import App from '../../../js/src/app.js'

onload = (evt) => {
    var worker = new Worker('/worker.js');
    let pipe = new WorkerPipe(worker);
    let app = new App(pipe, document.body);
}
