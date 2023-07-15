import { appAPI, onNewLogLines } from '$lib/api';
import { readable } from 'svelte/store';
import { Mutex } from 'async-mutex';
import type { CommandLogLines } from '$lib/types';
import { debounce } from 'lodash-es';

export function getLogLinesStore(commandId: number, initialCommandLogLines: CommandLogLines) {
  console.debug('Created log lines store', commandId);
  const store = readable(initialCommandLogLines as CommandLogLines, (set, update) => {
    console.debug('Subscribed to log lines store', commandId);
    const mutex = new Mutex();
    let lastLogId = 0;

    mutex.runExclusive(async () => {
      const log = await appAPI().getCommandLogLines(commandId);
      if (log.length > 0) lastLogId = log[log.length - 1].id;
      set(log);
    });

    const remove = onNewLogLines(
      commandId,
      debounce(
        () => {
          mutex.runExclusive(async () => {
            const newLog = await appAPI().getNewerCommandLines(commandId, lastLogId);
            if (newLog.length > 0) {
              lastLogId = newLog[newLog.length - 1].id;
              update((current) => [...current, ...newLog]);
            }
          });
        },
        100,
        { maxWait: 100 },
      ),
    );

    return () => {
      console.debug('Removed log lines store', commandId);
      remove();
    };
  });

  return store;
}
