-- CreateTable
CREATE TABLE "Command" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL,
    "cwd" TEXT NOT NULL,
    "command" TEXT NOT NULL,
    "order" TEXT NOT NULL,
    "lastRunResultType" TEXT,
    "lastRunCode" TEXT
);

-- CreateTable
CREATE TABLE "CommandLogLine" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "commandId" INTEGER NOT NULL,
    "source" INTEGER NOT NULL,
    "line" TEXT NOT NULL,
    "timestamp" REAL NOT NULL,
    CONSTRAINT "CommandLogLine_commandId_fkey" FOREIGN KEY ("commandId") REFERENCES "Command" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);

-- CreateIndex
CREATE UNIQUE INDEX "Command_order_key" ON "Command"("order");

-- CreateIndex
CREATE INDEX "CommandLogLine_commandId_timestamp_idx" ON "CommandLogLine"("commandId", "timestamp");

-- WAL Mode
PRAGMA journal_mode=WAL;