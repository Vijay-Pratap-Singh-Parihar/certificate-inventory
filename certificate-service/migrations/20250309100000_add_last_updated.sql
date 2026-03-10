-- Add last_updated for "Last Updated" column (default to creation time for existing rows).
ALTER TABLE certificates ADD COLUMN IF NOT EXISTS last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW();
