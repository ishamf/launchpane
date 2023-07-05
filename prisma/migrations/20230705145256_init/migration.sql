-- CreateTable
CREATE TABLE "Command" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL,
    "cwd" TEXT NOT NULL,
    "command" TEXT NOT NULL
);
