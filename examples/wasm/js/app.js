import WorkerPipe from '../../../js/src/worker_pipe.js'
import App from '../../../js/src/app.js'

import Worker from 'worker-loader!./worker.js';

onload = (evt) => {
    let pipe = new WorkerPipe(new Worker());
    let app = new App(pipe, document.body);
}
