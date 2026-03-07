-- Add an optional due_date column (ISO 8601 date string, e.g. "2025-12-31").
-- NULL means no due date. Existing tasks are unaffected.

ALTER TABLE tasks ADD COLUMN due_date TEXT;
