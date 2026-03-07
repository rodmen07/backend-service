-- Add an optional labels column (comma-separated list, e.g. "urgent,frontend,bug").
-- NULL means no labels. Existing tasks are unaffected.

ALTER TABLE tasks ADD COLUMN labels TEXT;
