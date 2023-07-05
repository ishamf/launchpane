/* eslint-disable @typescript-eslint/no-explicit-any */
import { contextBridge, ipcRenderer } from 'electron';
import type { Database } from './Database';
import type { WindowState } from '../types';

const exposedCommands = {
  getCommands: ((...args: any[]) =>
    ipcRenderer.invoke('command:getCommands', ...args)) as typeof Database.prototype.getCommands,
  addCommand: ((...args: any[]) =>
    ipcRenderer.invoke('command:addCommand', ...args)) as typeof Database.prototype.addCommand,
  updateCommand: ((...args: any[]) =>
    ipcRenderer.invoke(
      'command:updateCommand',
      ...args,
    )) as typeof Database.prototype.updateCommand,

  setWindowState: (state: WindowState) => ipcRenderer.invoke('window:setWindowState', state),
};

contextBridge.exposeInMainWorld('electronAPI', exposedCommands);

export type ElectronAPI = typeof exposedCommands;
