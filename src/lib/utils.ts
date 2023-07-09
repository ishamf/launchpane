import type { CommandObject } from './types';

export function getCommandDescriptor(command: CommandObject) {
  if (!command.command) return '...';
  if (!command.cwd) return command.command;

  const lastCWDDir = command.cwd.split('/').pop();

  return `${lastCWDDir}> ${command.command}`;
}

export function showCommandTitleWithMonospace(command: CommandObject) {
  return !!(command.command && !command.name);
}
