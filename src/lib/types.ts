import type { Prisma } from '@prisma/client';
import type { AppAPIType } from './api';

export enum WindowState {
  List = 0,
  Editing = 1,
}

export enum CommandLineSource {
  STDOUT = 1,
  STDERR = 2,
}

export enum CommandStatus {
  Stopped = 0,
  Running = 1,
}

export type CommandObject = NonNullable<Prisma.PromiseReturnType<AppAPIType['getCommand']>>;
