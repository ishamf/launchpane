import { PrismaClient } from '@prisma/client';
import { resolve } from 'path';

// @ts-expect-error untyped private api
import { MigrateDeploy } from '@prisma/migrate/dist/commands/MigrateDeploy';
import { writeFile } from 'fs/promises';

const dbURL = 'file:' + resolve(process.cwd(), 'prisma/app.db?connection_limit=1')

export async function initialiseDatabase() {
  await writeFile('.env', `DATABASE_URL="${dbURL}"`)
  console.log('Migrate deploy result:', await MigrateDeploy.new().parse([]));
}

export const prisma = new PrismaClient({
  datasources: {
    db: { url: dbURL },
  },
});

// export const prisma = new PrismaClient({ log: [{ emit: 'event', level: 'query' }] });

// prisma.$on('query', (e) => {
//   console.log('Query: ' + e.query);
//   console.log('Params: ' + e.params);
//   console.log('Duration: ' + e.duration + 'ms');
// });
