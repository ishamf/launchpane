/* eslint-disable @typescript-eslint/no-explicit-any */
import { contextBridge, ipcRenderer } from 'electron';

const exposedCommands = {
  invokeProxiedFunction: async (command: string, ...args: any[]): Promise<any> => {
    return ipcRenderer.invoke('invokeProxiedFunction', command, ...args);
  },
};

contextBridge.exposeInMainWorld('electronAPI', exposedCommands);

export type ElectronAPI = typeof exposedCommands;
