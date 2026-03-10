/**
 * Certificate type matching @certificate-service backend API.
 * GET /certificates/:id returns this shape.
 * Backend: domain/certificate.rs (id, subject, issuer, valid_from, expiration, signature_algorithm, san_entries).
 */
export interface Certificate {
  id: string;
  subject: string;
  issuer: string;
  /** Validity start (notBefore) ISO 8601 UTC. Present when parsed from PEM; null when loaded from list without re-parsing. */
  valid_from: string | null;
  /** ISO 8601 UTC datetime (notAfter, e.g. from chrono::DateTime<Utc>). */
  expiration: string;
  /** Signature algorithm (e.g. sha1WithRSAEncryption). Present when parsed from PEM; null otherwise. */
  signature_algorithm: string | null;
  /** Subject Alternative Names (DNS, IP, etc.). */
  san_entries: string[];
  /** Last time the record was updated (ISO 8601). */
  last_updated?: string | null;
}

/**
 * Generic paginated list response for GET /certificates?page=&limit=
 * Use when the backend implements a list endpoint.
 */
export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  limit: number;
  total_pages: number;
}

/**
 * Dashboard metrics (e.g. from GET /certificates/metrics or derived from list).
 * Backend may expose these when list/stats endpoints are added.
 */
export interface DashboardMetrics {
  total: number;
  expiring_soon: number; // e.g. next 30 days
}

/**
 * Certificate status derived from expiration (client-side).
 */
export type CertificateStatus = "Valid" | "Expiring Soon" | "Expired";

/**
 * Helper: derive status from expiration ISO string.
 */
export function getCertificateStatus(expirationIso: string): CertificateStatus {
  const exp = new Date(expirationIso);
  const now = new Date();
  const thirtyDaysFromNow = new Date(now);
  thirtyDaysFromNow.setDate(thirtyDaysFromNow.getDate() + 30);
  if (exp < now) return "Expired";
  if (exp <= thirtyDaysFromNow) return "Expiring Soon";
  return "Valid";
}
