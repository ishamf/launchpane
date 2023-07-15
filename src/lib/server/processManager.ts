import { CommandStatus, CommandLineSource, type CommandObject } from '../types';
import { spawn } from 'child_process';
import { homedir } from 'os';
import { createInterface } from 'readline';

import type { ChildProcessByStdio } from 'child_process';
import type { Writable, Readable } from 'stream';
import { prisma } from './db';
import { performance } from 'perf_hooks';
import { getShell } from './utils';
import { notifyCommandLogLineAdded, notifyCommandUpdated } from './notification';

const runningProcesses: { [id: number]: ChildProcessByStdio<Writable, Readable, Readable> } = {};

export function getCommandStatus(id: number): CommandStatus {
  return runningProcesses[id] ? CommandStatus.Running : CommandStatus.Stopped;
}

export async function runCommand(command: CommandObject) {
  if (runningProcesses[command.id]) {
    throw new Error(`Command ${command.id} is already running`);
  }

  const cwd = command.cwd ?? homedir();

  await prisma.commandLogLine.create({
    data: {
      commandId: command.id,
      source: CommandLineSource.INFO,
      line: `Running command`,
      timestamp: performance.timeOrigin + performance.now(),
    },
  });

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
    notifyCommandLogLineAdded(command.id);
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
    notifyCommandLogLineAdded(command.id);
  });

  let hasFinished = false;
  let endLine: string;

  childProcess.on('exit', async (code, signal) => {
    if (hasFinished) return;
    hasFinished = true;
    delete runningProcesses[command.id];

    const lastRunResultType = code == null ? 'signal' : 'code';
    const lastRunCode = lastRunResultType === 'code' ? code?.toString() : signal;

    endLine = `Command exited with ${lastRunResultType} ${lastRunCode}`;
    onExit();

    await prisma.command.update({
      where: { id: command.id },
      data: {
        lastRunResultType,
        lastRunCode,
      },
    });

    notifyCommandUpdated(command.id);
    notifyCommandLogLineAdded(command.id);
  });

  childProcess.on('error', async (err) => {
    if (hasFinished) return;
    hasFinished = true;
    delete runningProcesses[command.id];

    endLine = `Command exited with error ${err.message}`;
    onExit();

    await prisma.command.update({
      where: { id: command.id },
      data: {
        lastRunResultType: 'error',
        lastRunCode: err.message,
      },
    });

    notifyCommandUpdated(command.id);
    notifyCommandLogLineAdded(command.id);
  });

  // Only print the info line once all the output is saved
  type CB = (x: void) => void;
  let onStdoutClose: CB;
  let onStderrClose: CB;
  let onExit: CB;

  Promise.all([
    new Promise((r) => {
      onStdoutClose = r;
    }),
    new Promise((r) => {
      onStderrClose = r;
    }),
    new Promise((r) => {
      onExit = r;
    }),
  ]).then(() => {
    return prisma.commandLogLine.create({
      data: {
        commandId: command.id,
        source: CommandLineSource.INFO,
        line: endLine,
        timestamp: performance.timeOrigin + performance.now(),
      },
    });
  });

  // @ts-expect-error it's already assigned synchronously
  stdoutInterface.on('close', onStdoutClose);

  // @ts-expect-error it's already assigned synchronously
  stdErrInterface.on('close', onStderrClose);

  runningProcesses[command.id] = childProcess;
}

export function sendSignalToCommand(id: number, signal: NodeJS.Signals) {
  if (!runningProcesses[id]) {
    throw new Error(`Command ${id} is not running`);
  }

  return runningProcesses[id].kill(signal);
}
