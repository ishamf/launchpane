import * as bindingsApi from './generated/bindings';
import type { AppEventPayload } from './generated/events';
import { listen, type Event } from '@tauri-apps/api/event';

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
          try {
            // @ts-expect-error typechecked by proxy type
            const result = await bindingsApi[prop](...args);
            console.debug('Called API function', prop, args, result, performance.now() - prev);
            return result;
          } catch (e) {
            console.error('Failed to call function', e);
            throw e;
          }
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

export const onDataUpdate = (callback: (data: Event<AppEventPayload>) => void) => {
  const removePromise = listen<AppEventPayload>('change_event', callback);

  return () => removePromise.then((remove) => remove());
};

export const onCommandUpdate = (commandId: number, callback: () => void) => 
  onDataUpdate((data) => {
    if ('CommandUpdateEvent' in data.payload && data.payload.CommandUpdateEvent === commandId)
      callback();
  });

export const onNewLogLines = (commandId: number, callback: () => void) =>
  onDataUpdate((data) => {
    if ('CommandLogUpdateEvent' in data.payload && data.payload.CommandLogUpdateEvent === commandId)
      callback();
  });
