/*
  Warnings:

  - You are about to drop the column `createdAt` on the `CommandLogLine` table. All the data in the column will be lost.
  - Added the required column `timestamp` to the `CommandLogLine` table without a default value. This is not possible if the table is not empty.

*/
-- RedefineTables
PRAGMA foreign_keys=OFF;
CREATE TABLE "new_CommandLogLine" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "commandId" INTEGER NOT NULL,
    "source" INTEGER NOT NULL,
    "line" TEXT NOT NULL,
    "timestamp" REAL NOT NULL,
    CONSTRAINT "CommandLogLine_commandId_fkey" FOREIGN KEY ("commandId") REFERENCES "Command" ("id") ON DELETE RESTRICT ON UPDATE CASCADE
);
INSERT INTO "new_CommandLogLine" ("commandId", "id", "line", "source") SELECT "commandId", "id", "line", "source" FROM "CommandLogLine";
DROP TABLE "CommandLogLine";
ALTER TABLE "new_CommandLogLine" RENAME TO "CommandLogLine";
PRAGMA foreign_key_check;
PRAGMA foreign_keys=ON;
