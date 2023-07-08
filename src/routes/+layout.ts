import { appAPI } from '$lib/api';
import { WindowState } from '$lib/types';

export const ssr = false;

export async function load({depends}) {
  return {
    windowState: WindowState.List,
    commands: appAPI(depends).getCommands(),
  };
}
