/* eslint-disable @typescript-eslint/no-explicit-any */
import type { DataUpdateEvent } from '../types';
import { contextBridge, ipcRenderer } from 'electron';

const exposedCommands = {
  invokeProxiedFunction: async (command: string, ...args: any[]): Promise<any> => {
    return ipcRenderer.invoke('invokeProxiedFunction', command, ...args);
  },

  onDataUpdate: (callback: (data: DataUpdateEvent) => void) => {
    ipcRenderer.on('dataUpdate', (event, data) => {
      callback(data);
    });

    return () => {
      ipcRenderer.removeListener('dataUpdate', callback);
    };
  },
};

contextBridge.exposeInMainWorld('electronAPI', exposedCommands);

export type ElectronAPI = typeof exposedCommands;
