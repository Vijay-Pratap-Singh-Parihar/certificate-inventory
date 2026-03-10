-- Add PEM column for GET /certificates/:id/pem. Nullable for existing rows.
ALTER TABLE certificates ADD COLUMN IF NOT EXISTS pem TEXT;
