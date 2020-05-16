var addon = require('../native/index.node');

const {app, BrowserWindow, Menu} = require('electron');
process.env.ELECTRON_DISABLE_SECURITY_WARNINGS = true;


function createWindow () {
  const port = addon.run();
  console.log('PORT' + port);
  global.port = port;

  Menu.setApplicationMenu(null);
  const mainWindow = new BrowserWindow({
    width: 800,
    height: 600,
    webPreferences: {
      webSecurity: false,
      nodeIntegration: true
    }
  });
  mainWindow.loadFile('index.html');
  mainWindow.webContents.openDevTools()
}

app.whenReady().then(() => {
  createWindow();
  app.on('activate', function () {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  })
})

app.on('window-all-closed', function () {
  if (process.platform !== 'darwin') {
      app.quit();
  }
})
