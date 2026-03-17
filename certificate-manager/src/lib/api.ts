/**
 * API client for @certificate-service.
 * All API calls use NEXT_PUBLIC_CERTIFICATE_SERVICE_URL so the static export (e.g. GitHub Pages with basePath) hits the backend, not relative paths.
 * List/metrics endpoints may be added to the backend; until then, list returns empty and metrics zeros.
 */

import type { Certificate, DashboardMetrics, PaginatedResponse } from "@/types/certificate";
import { apiFetch, ApiError } from "./fetcher";

/** Backend API base URL (e.g. https://certificate-inventory.onrender.com). Set NEXT_PUBLIC_CERTIFICATE_SERVICE_URL at build time for static export. */
export const API_BASE =
  process.env.NEXT_PUBLIC_CERTIFICATE_SERVICE_URL ?? "http://localhost:3001";

export { apiFetch, ApiError } from "./fetcher";

const ALLOW_SELF_SIGNED = process.env.NODE_ENV === "development" && process.env.CERTIFICATE_SERVICE_ALLOW_SELF_SIGNED === "true";

/** GET /certificates/:id */
export async function getCertificateById(id: string): Promise<Certificate> {
  return apiFetch<Certificate>(`/certificates/${encodeURIComponent(id)}`, {
    allowSelfSigned: ALLOW_SELF_SIGNED,
  });
}

/** GET /certificates/:id/pem — returns { pem: string } for download. */
export async function getCertificatePem(id: string): Promise<{ pem: string }> {
  return apiFetch<{ pem: string }>(`/certificates/${encodeURIComponent(id)}/pem`, {
    allowSelfSigned: ALLOW_SELF_SIGNED,
  });
}

/** POST /certificates — upload PEM; backend parses and stores. Returns created certificate. */
export async function createCertificate(pem: string): Promise<Certificate> {
  return apiFetch<Certificate>("/certificates", {
    method: "POST",
    body: JSON.stringify({ pem }),
    allowSelfSigned: ALLOW_SELF_SIGNED,
  });
}

/**
 * GET /certificates?page=&limit=&status=
 * When the backend implements this endpoint, it should return PaginatedResponse<Certificate>.
 * Until then, we return a placeholder (empty list) so the UI can render.
 */
export async function getCertificates(params: {
  page?: number;
  limit?: number;
  status?: string;
}): Promise<PaginatedResponse<Certificate>> {
  const search = new URLSearchParams();
  if (params.page != null) search.set("page", String(params.page));
  if (params.limit != null) search.set("limit", String(params.limit));
  if (params.status != null && params.status !== "All Statuses") search.set("status", params.status);
  const query = search.toString();
  const path = query ? `/certificates?${query}` : "/certificates";

  const emptyResult: PaginatedResponse<Certificate> = {
    items: [],
    total: 0,
    page: params.page ?? 1,
    limit: params.limit ?? 10,
    total_pages: 0,
  };

  try {
    const data = await apiFetch<PaginatedResponse<Certificate>>(path, {
      allowSelfSigned: ALLOW_SELF_SIGNED,
    });
    return data;
  } catch (e) {
    // Backend may not have list endpoint (404/501/405) or service unreachable (network error).
    // Return empty paginated result so the UI can render; client can retry via SWR.
    const status = e instanceof ApiError ? e.status : 0;
    if (status === 404 || status === 501 || status === 405 || status === 0) {
      return emptyResult;
    }
    throw e;
  }
}

/**
 * GET /certificates/metrics or similar.
 * When the backend adds dashboard metrics, point this to the real endpoint.
 * Until then, derive from list or return zeros.
 */
export async function getDashboardMetrics(): Promise<DashboardMetrics> {
  try {
    const data = await apiFetch<DashboardMetrics>("/certificates/metrics", {
      allowSelfSigned: ALLOW_SELF_SIGNED,
    });
    return data;
  } catch {
    // No metrics endpoint yet; return zeros (inventory page can show 0/0 or derive from list).
    return { total: 0, expiring_soon: 0 };
  }
}
