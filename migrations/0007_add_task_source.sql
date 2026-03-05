-- Add a source column to distinguish manually created tasks from AI-generated ones.
-- Values: 'manual' (default), 'ai_generated'.
-- Existing tasks are treated as manually created.

ALTER TABLE tasks ADD COLUMN source TEXT NOT NULL DEFAULT 'manual';
