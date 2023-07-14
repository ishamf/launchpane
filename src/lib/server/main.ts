import { join } from 'path';
import { app, BrowserWindow, ipcMain } from 'electron';
import * as API from './api';
import { setLatestIPCEvent } from './utils';
import type {} from '../../app.d.ts';
import { addDataUpdateListener } from './notification';

const createWindow = () => {
  const win = new BrowserWindow({
    width: 300,
    height: 650,
    webPreferences: {
      preload: join(__dirname, 'preload.js'),
    },

    autoHideMenuBar: true,
  });

  win.loadURL('http://localhost:5173');

  win.webContents.openDevTools();

  const remove = addDataUpdateListener((data) => {
    win.webContents.send('dataUpdate', data);
  });

  win.on('close', () => {
    remove();
  });
};

app.whenReady().then(() => {
  ipcMain.handle('invokeProxiedFunction', (event, fnName: string, ...args: unknown[]) => {
    setLatestIPCEvent(event);
    // @ts-expect-error we typecheck this in the clientside proxy
    const promise = API[fnName](...args);
    setLatestIPCEvent(null);

    return promise;
  });

  createWindow();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit();
});
