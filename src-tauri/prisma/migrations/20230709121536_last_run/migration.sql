-- RedefineTables
PRAGMA foreign_keys=OFF;
CREATE TABLE "new_Command" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL,
    "cwd" TEXT NOT NULL,
    "command" TEXT NOT NULL,
    "lastRunResultType" TEXT,
    "lastRunCode" TEXT
);
INSERT INTO "new_Command" ("command", "cwd", "id", "lastRunCode", "lastRunResultType", "name") SELECT "command", "cwd", "id", "lastRunCode", "lastRunResultType", "name" FROM "Command";
DROP TABLE "Command";
ALTER TABLE "new_Command" RENAME TO "Command";
PRAGMA foreign_key_check;
PRAGMA foreign_keys=ON;
