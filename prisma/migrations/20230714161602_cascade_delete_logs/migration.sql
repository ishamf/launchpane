-- RedefineTables
PRAGMA foreign_keys=OFF;
CREATE TABLE "new_CommandLogLine" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "commandId" INTEGER NOT NULL,
    "source" INTEGER NOT NULL,
    "line" TEXT NOT NULL,
    "timestamp" REAL NOT NULL,
    CONSTRAINT "CommandLogLine_commandId_fkey" FOREIGN KEY ("commandId") REFERENCES "Command" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);
INSERT INTO "new_CommandLogLine" ("commandId", "id", "line", "source", "timestamp") SELECT "commandId", "id", "line", "source", "timestamp" FROM "CommandLogLine";
DROP TABLE "CommandLogLine";
ALTER TABLE "new_CommandLogLine" RENAME TO "CommandLogLine";
CREATE INDEX "CommandLogLine_commandId_timestamp_idx" ON "CommandLogLine"("commandId", "timestamp");
PRAGMA foreign_key_check;
PRAGMA foreign_keys=ON;
