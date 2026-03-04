PRAGMA foreign_keys = OFF;

BEGIN TRANSACTION;

CREATE TABLE tasks_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    completed INTEGER NOT NULL DEFAULT 0,
    difficulty INTEGER NOT NULL DEFAULT 1 CHECK (difficulty >= 1 AND difficulty <= 6),
    goal TEXT
);

INSERT INTO tasks_new (id, title, completed, difficulty, goal)
SELECT
    id,
    title,
    completed,
    CASE
        WHEN difficulty < 1 THEN 1
        WHEN difficulty > 6 THEN 6
        ELSE difficulty
    END,
    goal
FROM tasks;

DROP TABLE tasks;
ALTER TABLE tasks_new RENAME TO tasks;

COMMIT;

PRAGMA foreign_keys = ON;
