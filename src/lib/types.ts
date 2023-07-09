import type { Prisma } from "@prisma/client";
import type { AppAPIType } from "./api";

export enum WindowState {
  List = 0,
  Editing = 1,
}

export type CommandObject = NonNullable<Prisma.PromiseReturnType<AppAPIType['getCommand']>>;
