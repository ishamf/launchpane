export function load({ params }) {
  return {
    command: electronAPI.getCommand(parseInt(params.id)),
  };
}
