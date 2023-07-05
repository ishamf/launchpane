export async function load() {
  return {
    commands: await electronAPI.getCommands(),
  };
}
