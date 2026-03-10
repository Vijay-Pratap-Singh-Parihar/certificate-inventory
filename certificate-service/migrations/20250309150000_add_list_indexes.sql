-- Indexes for list/filter/sort performance (50k+ certificates).
-- expiration index may already exist from initial migration; IF NOT EXISTS keeps idempotent.
CREATE INDEX IF NOT EXISTS idx_certificates_expiration ON certificates (expiration);
CREATE INDEX IF NOT EXISTS idx_certificates_issuer ON certificates (issuer);
CREATE INDEX IF NOT EXISTS idx_certificates_last_updated ON certificates (last_updated);
