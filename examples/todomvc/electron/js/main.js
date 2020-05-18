import addon from 'node-loader!../native/index.node';

import showDialog from '../../../../js/src/electron_dialogs';

const {app, BrowserWindow, Menu, ipcMain} = require('electron');
process.env.ELECTRON_DISABLE_SECURITY_WARNINGS = true;


function createWindow () {
  const port = addon.run();
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
  mainWindow.loadURL(`file://${__dirname}/index.html`);
  if (process.env.NODE_ENV === "development") {
    mainWindow.webContents.openDevTools();
  }
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

ipcMain.on('dialog', (event, arg) => {
  showDialog(arg, (result) => {
    event.reply('dialog', result);
  });
});