import { readable } from 'svelte/store';
import { appAPI, onCommandUpdate, onNewLogLines } from './api';
import { Mutex } from 'async-mutex';
import type { ProcessStatus } from './types';
import { throttle } from 'lodash-es';

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

async function delay(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export function createRecentActivityStore(commandId: number) {
  return readable(false, (set) => {
    console.debug('Subscribed to command recent activity store', commandId);

    let isAlreadyRunning = false;

    const remove = onNewLogLines(commandId, async () => {
      if (isAlreadyRunning) return;
      isAlreadyRunning = true;
      set(true);
      await delay(100);
      set(false);
      await delay(100);
      isAlreadyRunning = false;
    });
    return () => {
      console.debug('Removed command recent activity store', commandId);
      remove();
    };
  });
}
