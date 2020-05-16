
var remote = require('electron').remote; 
const port = remote.getGlobal('port');
console.log(port)