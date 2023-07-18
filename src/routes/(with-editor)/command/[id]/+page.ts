import { appAPI } from '$lib/api';
import { error } from '@sveltejs/kit';

export async function load({ params, depends }) {
  const command = await appAPI(depends).getCommand(parseInt(params.id));

  if (!command) {
    throw error(404, 'Command not found');
  }
  const processStatus = await appAPI(depends).getProcessStatus(command.id);

  const initialCommandLogLines = await appAPI(depends).getCommandLogLines(command.id);

  return {
    command,
    processStatus,
    initialCommandLogLines,
  };
}
