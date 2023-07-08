import type { IpcMainInvokeEvent } from 'electron';

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
