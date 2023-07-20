export type { Command, CommandLogLine, ProcessStatus } from './generated/bindings';

export enum WindowState {
  List = 0,
  Editing = 1,
}

export enum CommandLineSource {
  STDOUT = 1,
  STDERR = 2,

  INFO = 3,
}



export type DataUpdateEvent =
  | {
      type: 'command';
      id: number;
    }
  | {
      type: 'commandLogLine';
      id: number;
    };
