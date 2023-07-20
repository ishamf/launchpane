import { readable } from 'svelte/store';
import { appAPI, onCommandUpdate } from './api';
import { Mutex } from 'async-mutex';
import type { ProcessStatus } from './types';

export function createCommandStatusStore(commandId: number) {
  return readable<ProcessStatus>('Stopped', (set) => {
    const mutex = new Mutex();
    console.debug('Subscribed to command status store', commandId);
    function updateStatus() {
      mutex.runExclusive(async () => {
        const status = await appAPI().getProcessStatus(commandId);
        set(status);
      });
    }

    updateStatus();

    const remove = onCommandUpdate(commandId, () => {
      updateStatus();
    });
    return () => {
      console.debug('Removed command status store', commandId);
      remove();
    };
  });
}
