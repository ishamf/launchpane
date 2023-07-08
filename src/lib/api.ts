import type * as API from './server/api';
import { stringify } from 'devalue';



export const appAPI = (depends?:  (id: string) => void) =>
  new Proxy(
    {},
    {
      get: (target, prop) => {
        return (...args: unknown[]) => {
          depends?.('electronAPI:' + stringify({ prop, args }));
          return electronAPI.invokeProxiedFunction(prop as string, ...args);
        };
      },
    },
  ) as typeof API;
