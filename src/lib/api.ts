import type * as API from './server/api';
import type { DataUpdateEvent } from './types';

export type AppAPIType = typeof API;

export const appAPI = (depends?: (id: string) => void) =>
  // @ts-expect-error testing only
  ({
    setWindowState() {
      console.log('pass');
    },
    getCommands() {
      return [];
    },
    getPlatformDetails() {
      return {
        pathSeparator: '/',
      }
    }
  } as typeof API);

export const apiUrls = {
  getCommands: () => 'electronAPI:getCommands',
  getCommand: (id: number) => `electronAPI:getCommand:${id}`,
};

const internalDependsString: {
  [k in keyof AppAPIType]?: (...args: Parameters<AppAPIType[k]>) => string;
} = apiUrls;

export const onDataUpdate = (callback: (data: DataUpdateEvent) => void) => () => {
  console.log('pass');
};

export const onNewLogLines = (commandId: number, callback: () => void) =>
  onDataUpdate((data) => {
    if (data.type === 'commandLogLine' && data.id === commandId) callback();
  });
