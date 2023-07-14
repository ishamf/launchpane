import { appAPI, onNewLogLines } from '$lib/api';
import { readable } from 'svelte/store';
import { Mutex } from 'async-mutex';
import type { CommandLogLines } from '$lib/types';

export function getNewLogLinesStore(commandId: number, lastLogId: number) {
  console.debug('Created new log lines store', commandId, lastLogId);
  let latestLogId = lastLogId;
  const mutex = new Mutex();
  const store = readable([] as CommandLogLines, (set, update) => {
    console.debug('Subscribed to log lines store', commandId, lastLogId);

    const remove = onNewLogLines(commandId, () => {
      mutex.runExclusive(async () => {
        const newCommandLines = await appAPI().getNewerCommandLines(commandId, latestLogId);
        if (newCommandLines.length > 0) {
          latestLogId = newCommandLines[newCommandLines.length - 1].id;
          update((current) => [...current, ...newCommandLines]);
        }
      });
    });

    return () => {
        console.debug('Removed log lines store', commandId, lastLogId);
        remove();
      }
  });

  return store;
}
