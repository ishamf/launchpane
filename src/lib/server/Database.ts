import type { PrismaClient, Prisma } from '@prisma/client';

export class Database {
  client: PrismaClient;

  constructor(client: PrismaClient) {
    this.client = client;
  }

  async getCommands() {
    return this.client.command.findMany();
  }

  async addCommand(data: Prisma.CommandCreateInput) {
    return this.client.command.create({
      data,
    });
  }

  async updateCommand(id: number, data: Prisma.CommandUpdateInput) {
    return this.client.command.update({
      where: { id },
      data,
    });
  }
}

export function getDBFunctionMap(db: Database) {
  return {
    'command:getCommands': db.getCommands.bind(db) as typeof db.getCommands,
    'command:addCommand': db.addCommand.bind(db) as typeof db.addCommand,
    'command:updateCommand': db.updateCommand.bind(db) as typeof db.updateCommand,
  };
}
