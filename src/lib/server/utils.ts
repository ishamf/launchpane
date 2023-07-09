import type { IpcMainInvokeEvent } from 'electron';

import { platform } from 'os';

let currentIPCEvent = null as null | IpcMainInvokeEvent;

export function getIPCEvent(): IpcMainInvokeEvent {
  if (!currentIPCEvent) {
    throw new Error('This function must be called immediately when the function starts');
  }
  return currentIPCEvent;
}

export function setLatestIPCEvent(event: IpcMainInvokeEvent | null) {
  currentIPCEvent = event;
}

export function getShell(): string | boolean {
  // Seems Dash doesn't propagate signals
  if (platform() === 'linux') return '/bin/bash';

  return true;
}
