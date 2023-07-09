import type * as API from './server/api';

export type AppAPIType = typeof API;

export const appAPI = (depends?: (id: string) => void) =>
  new Proxy(
    {},
    {
      get: <P extends keyof AppAPIType>(target: unknown, prop: P) => {
        return (...args: Parameters<AppAPIType[P]>) => {
          if (depends) {
            const dependsStringFn = internalDependsString[prop];
            if (dependsStringFn) depends(dependsStringFn(...args));
          }

          return electronAPI.invokeProxiedFunction(prop, ...args);
        };
      },
    },
  ) as typeof API;

export const apiUrls = {
  getCommands: () => 'electronAPI:getCommands',
  getCommand: (id: number) => `electronAPI:getCommand:${id}`,
};

const internalDependsString: {
  [k in keyof AppAPIType]?: (...args: Parameters<AppAPIType[k]>) => string;
} = apiUrls;
