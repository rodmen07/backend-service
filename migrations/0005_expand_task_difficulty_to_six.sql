-- Expand difficulty constraint from 1-5 to 1-6.
-- PostgreSQL supports DROP CONSTRAINT / ADD CONSTRAINT without table rebuild.

ALTER TABLE tasks DROP CONSTRAINT IF EXISTS tasks_difficulty_check;

UPDATE tasks SET difficulty = LEAST(GREATEST(difficulty, 1), 6);

ALTER TABLE tasks ADD CONSTRAINT tasks_difficulty_check
    CHECK (difficulty >= 1 AND difficulty <= 6);
