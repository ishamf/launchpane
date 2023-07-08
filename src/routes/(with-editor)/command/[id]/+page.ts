import { appAPI } from '$lib/api';

export function load({ params, depends }) {
  return {
    command: appAPI(depends).getCommand(parseInt(params.id)),
  };
}
