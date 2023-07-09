import type { ElectronAPI, WindowState } from './lib/server/types';

// See https://kit.svelte.dev/docs/types#app
// for information about these interfaces
declare global {
  namespace App {
    // interface Error {}
    // interface Locals {}
    interface PageData {
      windowState: WindowState;
      command?: CommandObject;
    }
    // interface Platform {}
  }

  const electronAPI: ElectronAPI;
}

export {};
