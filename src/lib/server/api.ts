import type { Prisma } from '@prisma/client';
import { prisma } from './db';
import { BrowserWindow } from 'electron';
import { WindowState, CommandStatus } from '../types';
import { getIPCEvent } from './utils';
import { homedir } from 'os';
import { sep } from 'path';
import {
  getCommandStatus,
  runCommand as runCommandinManager,
  sendSignalToCommand as sendSignalToCommandInManager,
} from './processManager';
import { notifyCommandUpdated } from './notification';

export async function getCommands() {
  return (await prisma.command.findMany()).map((x) => ({ ...x, status: getCommandStatus(x.id) }));
}

export async function getCommand(id: number) {
  const commandResult = await prisma.command.findFirst({ where: { id } });

  if (!commandResult) return null;

  return { ...commandResult, status: getCommandStatus(id) };
}

export async function runCommand(id: number) {
  if (getCommandStatus(id) === CommandStatus.Running) {
    throw new Error('Command is already running');
  }

  const command = await getCommand(id);

  if (!command) {
    throw new Error('Command not found');
  }

  await runCommandinManager(command);

  notifyCommandUpdated(id);
}

export async function sendSignalToCommand(id: number, signal: NodeJS.Signals) {
  console.log('gcs', getCommandStatus(id), { signal });
  if (getCommandStatus(id) === CommandStatus.Stopped) return;

  return sendSignalToCommandInManager(id, signal);
}

export async function getCommandLogLines(id: number) {
  return (
    await prisma.commandLogLine.findMany({
      where: { commandId: id },
      orderBy: { timestamp: 'desc' },
      take: 1000,
    })
  ).reverse();
}

export async function getNewerCommandLines(commandId: number, lastLogId: number) {
  if (!lastLogId) {
    return getCommandLogLines(commandId);
  }

  return prisma.commandLogLine.findMany({
    where: { commandId },
    orderBy: { timestamp: 'asc' },
    cursor: { id: lastLogId },
    skip: 1,
    take: 1000,
  });
}

export async function addCommand(data: Omit<Prisma.CommandCreateInput, 'cwd'> & { cwd?: string }) {
  const dataWithDefaults = { ...data, cwd: data.cwd ?? homedir() };
  const result = await prisma.command.create({
    data: dataWithDefaults,
  });
  notifyCommandUpdated(result.id);
  return result;
}

export async function updateCommand(id: number, data: Prisma.CommandUpdateInput) {
  const result = await prisma.command.update({
    where: { id },
    data,
  });
  notifyCommandUpdated(id);
  return result;
}

export async function deleteCommand(id: number) {
  const result = await prisma.command.delete({ where: { id } });
  notifyCommandUpdated(id);
  return result;
}

export async function setWindowState(state: WindowState) {
  const event = getIPCEvent();
  const window = BrowserWindow.getAllWindows().find((x) => x.webContents === event.sender);

  if (!window) {
    console.error('No window found');
    return;
  }

  switch (state) {
    case WindowState.List:
      window.setContentSize(300, 650);
      break;
    case WindowState.Editing:
      window.setContentSize(1280, 650);
      break;
  }
}

export async function getPlatformDetails() {
  return {
    pathSeparator: sep,
  };
}
