import { invalidate } from '$app/navigation';
import { onDataUpdate, apiUrls } from '$lib/api';

export function setupCommandInvalidation() {
  onDataUpdate((data) => {
    if (data.type === 'command') {
      invalidate(apiUrls.getCommands());
      invalidate(apiUrls.getCommand(data.id));
    }
  });
}

setupCommandInvalidation();
