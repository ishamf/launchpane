import { appAPI, type AppAPIType } from './api';

let platformDetails: Awaited<ReturnType<AppAPIType['getPlatformDetails']>>;

export async function loadPlatformData() {
  platformDetails = await appAPI().getPlatformDetails();
}

export function getPlatformDetails() {
  if (!platformDetails) throw new Error('Platform details not initialized');

  return {
    pathSeparator: platformDetails.path_separator,
  };
}
