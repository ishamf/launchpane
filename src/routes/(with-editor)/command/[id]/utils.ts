import { appAPI, onNewLogLines } from '$lib/api';
import { writable } from 'svelte/store';
import { Mutex } from 'async-mutex';
import type { CommandLogLine } from '$lib/types';
import { debounce } from 'lodash-es';

export function getLogLinesStore(commandId: number, initialCommandLogLines: CommandLogLine[]) {
  console.debug('Created log lines store', commandId);
  let firstLogId = 0;

  const mutex = new Mutex();

  const store = writable(initialCommandLogLines as CommandLogLine[], (set, update) => {
    console.debug('Subscribed to log lines store', commandId);

    let lastLogId = 0;

    function updateWithNewLogs() {
      mutex.runExclusive(async () => {
        const newLog = await appAPI().getNewerCommandLogLines(commandId, lastLogId);
        if (newLog.length > 0) {
          if (firstLogId === 0) firstLogId = newLog[0].id;
          lastLogId = newLog[newLog.length - 1].id;
          update((current) => [...current, ...newLog]);
        }
      });
    }

    set(initialCommandLogLines as CommandLogLine[]);

    if (initialCommandLogLines.length > 0) {
      firstLogId = initialCommandLogLines[0].id;
      lastLogId = initialCommandLogLines[initialCommandLogLines.length - 1].id;
      updateWithNewLogs();
    }

    const remove = onNewLogLines(
      commandId,
      debounce(() => updateWithNewLogs(), 100, { maxWait: 100 }),
    );

    return () => {
      console.debug('Removed log lines store', commandId);
      remove();
    };
  });

  function loadMore() {
    console.log('Loading more logs, from', firstLogId)
    if (!firstLogId) return;

    mutex.runExclusive(async () => {
      const newLog = await appAPI().getOlderCommandLogLines(commandId, firstLogId);
      if (newLog.length > 0) {
        if (firstLogId === 0) firstLogId = newLog[0].id;
        firstLogId = newLog[0].id;
        store.update((current) => [...newLog, ...current]);
      }
    });
  }

  return {
    subscribe: store.subscribe,
    loadMore,
  };
}
