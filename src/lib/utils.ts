import { getPlatformDetails } from './platformData';
import type { Command } from './types';

export function getCommandDescriptor(command: Command) {
  if (!command.command) return '...';
  if (!command.cwd) return command.command;

  const lastCWDDir = command.cwd.split(getPlatformDetails().pathSeparator).pop();

  return `${lastCWDDir}> ${command.command}`;
}

export function showCommandTitleWithMonospace(command: Command) {
  return !!(command.command && !command.name);
}
