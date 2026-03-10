-- Certificate table: id, subject, issuer, expiration, san_entries (array for SANs).
CREATE TABLE IF NOT EXISTS certificates (
    id          VARCHAR(36) PRIMARY KEY,
    subject     TEXT        NOT NULL,
    issuer      TEXT        NOT NULL,
    expiration  TIMESTAMPTZ NOT NULL,
    san_entries TEXT[]      NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_certificates_expiration ON certificates (expiration);
