ALTER TABLE tasks
ADD COLUMN difficulty INTEGER NOT NULL DEFAULT 1 CHECK (difficulty >= 1 AND difficulty <= 5);
