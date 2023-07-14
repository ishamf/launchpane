import { appAPI } from '$lib/api';
import { loadPlatformData } from '$lib/platformData';
import { WindowState } from '$lib/types';

export const ssr = false;

export async function load({ depends }) {
  await loadPlatformData();

  return {
    windowState: WindowState.List,
    commands: appAPI(depends).getCommands(),
  };
}
