import WorkerPipe from '../../../../js/worker_pipe.js'
import App from '../../../../js/app.js'

onload = (evt) => {
    var worker = new Worker('/worker.js');
    let pipe = new WorkerPipe(worker);
    let app = new App(pipe, document.body);
}
