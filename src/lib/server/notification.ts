import type { DataUpdateEvent } from '../types';

const listeners = new Set<(data: DataUpdateEvent) => void>();

export function notifyDataUpdate(event: DataUpdateEvent) {
  for (const listener of listeners) {
    listener(event);
  }
}

export function addDataUpdateListener(listener: (data: DataUpdateEvent) => void) {
  listeners.add(listener);

  return () => {
    listeners.delete(listener);
  };
}

export function notifyCommandUpdated(id: number) {
  notifyDataUpdate({
    type: 'command',
    id,
  });
}

export function notifyCommandLogLineAdded(id: number) {
  notifyDataUpdate({
    type: 'commandLogLine',
    id,
  });
}
