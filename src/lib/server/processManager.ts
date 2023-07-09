import { CommandStatus, CommandLineSource, type CommandObject } from '../types';
import { spawn } from 'child_process';
import { homedir } from 'os';
import { createInterface } from 'readline';

import type { ChildProcessByStdio } from 'child_process';
import type { Writable, Readable } from 'stream';
import { prisma } from './db';
import { performance } from 'perf_hooks';
import { getShell } from './utils';

const runningProcesses: { [id: number]: ChildProcessByStdio<Writable, Readable, Readable> } = {};

export function getCommandStatus(id: number): CommandStatus {
  return runningProcesses[id] ? CommandStatus.Running : CommandStatus.Stopped;
}

export function runCommand(command: CommandObject) {
  if (runningProcesses[command.id]) {
    throw new Error(`Command ${command.id} is already running`);
  }

  const cwd = command.cwd ?? homedir();

  const childProcess = spawn(command.command, { cwd, shell: getShell() });

  // console.log({ pid: childProcess.pid });

  const stdoutInterface = createInterface({ input: childProcess.stdout, crlfDelay: Infinity });
  const stdErrInterface = createInterface({ input: childProcess.stderr, crlfDelay: Infinity });

  stdoutInterface.on('line', async (line) => {
    // console.log({ line });
    await prisma.commandLogLine.create({
      data: {
        commandId: command.id,
        source: CommandLineSource.STDOUT,
        line,
        timestamp: performance.timeOrigin + performance.now(),
      },
    });
  });

  stdErrInterface.on('line', async (line) => {
    // console.log({ line });
    await prisma.commandLogLine.create({
      data: {
        commandId: command.id,
        source: CommandLineSource.STDERR,
        line,
        timestamp: performance.timeOrigin + performance.now(),
      },
    });
  });

  childProcess.on('exit', (code, signal) => {
    delete runningProcesses[command.id];

    const lastRunResultType = code == null ? 'signal' : 'code';
    const lastRunCode = lastRunResultType === 'code' ? code?.toString() : signal;

    prisma.command.update({
      where: { id: command.id },
      data: {
        lastRunResultType,
        lastRunCode,
      },
    });
  });

  runningProcesses[command.id] = childProcess;
}

export function sendSignalToCommand(id: number, signal: NodeJS.Signals) {
  if (!runningProcesses[id]) {
    throw new Error(`Command ${id} is not running`);
  }

  return runningProcesses[id].kill(signal);
}
