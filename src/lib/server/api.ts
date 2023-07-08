import type { Prisma } from '@prisma/client';
import { prisma } from './db';
import { BrowserWindow } from 'electron';
import { WindowState } from '../types';
import { getIPCEvent } from './utils';

export async function getCommands() {
  return prisma.command.findMany();
}

export async function getCommand(id: number) {
  return prisma.command.findFirst({ where: { id } });
}

export async function addCommand(data: Prisma.CommandCreateInput) {
  return prisma.command.create({
    data,
  });
}

export async function updateCommand(id: number, data: Prisma.CommandUpdateInput) {
  return prisma.command.update({
    where: { id },
    data,
  });
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
      window.setSize(300, 650);
      break;
    case WindowState.Editing:
      window.setSize(800, 650);
      break;
  }
}
