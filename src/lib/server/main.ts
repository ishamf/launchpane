import { PrismaClient } from '@prisma/client';
import { join } from 'path';
import { app, BrowserWindow, ipcMain } from 'electron';
import { Database, getDBFunctionMap } from './Database';
import { WindowState } from '../types';

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
};

app.whenReady().then(() => {
  const prisma = new PrismaClient();

  const db = new Database(prisma);

  for (const [command, fn] of Object.entries(getDBFunctionMap(db))) {
    // @ts-expect-error as we will typecheck in renderer
    ipcMain.handle(command, (event, ...args) => fn(...args));
  }

  ipcMain.handle('window:setWindowState', (event, state: WindowState) => {
    const window = BrowserWindow.getAllWindows().find((x) => x.webContents === event.sender);
    if (!window) {
      console.error('No window found');
      return;
    }

    switch (state) {
      case WindowState.List:
        window.setSize(300, 650);
        break;
      case WindowState.Editing:
        window.setSize(800, 650);
        break;
    }
  });

  createWindow();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit();
});
