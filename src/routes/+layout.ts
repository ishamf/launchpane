import { WindowState } from '$lib/types';

export const ssr = false;

export async function load() {
  return {
    windowState: WindowState.List,
    commands: await electronAPI.getCommands(),
  };
}
