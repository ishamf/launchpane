export type { Command } from './generated/bindings';
import type { CommandLogLine } from './generated/bindings';

export enum WindowState {
  List = 0,
  Editing = 1,
}

export enum CommandLineSource {
  STDOUT = 1,
  STDERR = 2,

  INFO = 3,
}

export enum CommandStatus {
  Stopped = 0,
  Running = 1,
}

export type CommandLogLines = CommandLogLine[];

export type DataUpdateEvent =
  | {
      type: 'command';
      id: number;
    }
  | {
      type: 'commandLogLine';
      id: number;
    };
