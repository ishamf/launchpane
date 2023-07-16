import * as bindingsApi from './bindings';
import type { DataUpdateEvent } from './types';

export type AppAPIType = typeof bindingsApi;

export const appAPI = (depends?: (id: string) => void) =>
  new Proxy(
    {},
    {
      get: <P extends keyof AppAPIType>(target: unknown, prop: P) => {
        return async (...args: Parameters<AppAPIType[P]>) => {
          if (depends) {
            const dependsStringFn = internalDependsString[prop];
            if (dependsStringFn) depends(dependsStringFn(...args));
          }

          console.debug('Calling API function', prop, args);
          const prev = performance.now();
          // @ts-expect-error typechecked by proxy type
          const result = await bindingsApi[prop](...args);
          console.debug('Called API function', prop, args, result, performance.now() - prev);
          return result;
        };
      },
    },
  ) as AppAPIType;

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
