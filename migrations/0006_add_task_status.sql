-- Add a status column for Kanban board workflow.
-- Values: 'todo', 'doing', 'done'. Default is 'todo'.
-- Existing completed tasks are migrated to 'done'.

ALTER TABLE tasks ADD COLUMN status TEXT NOT NULL DEFAULT 'todo';

UPDATE tasks SET status = 'done' WHERE completed = 1;
