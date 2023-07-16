import { invalidate } from '$app/navigation';
import { onDataUpdate, apiUrls } from '$lib/api';

export function setupCommandInvalidation() {
  onDataUpdate((data) => {
    console.log('Received event', data);
    if ('CommandUpdateEvent' in data.payload) {
      invalidate(apiUrls.getCommands());
      invalidate(apiUrls.getCommand(data.payload.CommandUpdateEvent));
    }
  });
}

setupCommandInvalidation();
