/**
 * Secure fetcher for @certificate-service API.
 * Supports self-signed TLS in local dev (SSR) and configurable base URL.
 * For mTLS: configure client certs via env (e.g. NODE_EXTRA_CA_CERTS) or extend this utility.
 *
 * Base URL resolution:
 * - Server (SSR/Docker): CERTIFICATE_SERVICE_URL (e.g. http://certificate-service:3001)
 * - Browser: NEXT_PUBLIC_CERTIFICATE_SERVICE_URL (e.g. http://localhost:3001)
 * - Local dev (non-Docker): defaults to http://localhost:3001
 */

const DEFAULT_BASE = "http://localhost:3001";

/** Resolved API base URL: server prefers CERTIFICATE_SERVICE_URL so Docker uses service hostname. */
function getBaseUrl(): string {
  if (typeof window !== "undefined") {
    return process.env.NEXT_PUBLIC_CERTIFICATE_SERVICE_URL ?? DEFAULT_BASE;
  }
  const API_BASE_URL =
    process.env.CERTIFICATE_SERVICE_URL ||
    process.env.NEXT_PUBLIC_CERTIFICATE_SERVICE_URL ||
    DEFAULT_BASE;
  return API_BASE_URL;
}

export type FetcherOptions = RequestInit & {
  /** Allow self-signed TLS (SSR only, development). Uses undici Agent when true. */
  allowSelfSigned?: boolean;
};

/**
 * Fetch from the certificate-service API.
 * On the server, when allowSelfSigned is true (e.g. in dev), uses an HTTPS agent that does not reject self-signed certs.
 */
export async function apiFetch<T = unknown>(
  path: string,
  options: FetcherOptions = {}
): Promise<T> {
  const { allowSelfSigned, ...init } = options;
  const base = getBaseUrl();
  const url = path.startsWith("http") ? path : `${base.replace(/\/$/, "")}${path.startsWith("/") ? path : `/${path}`}`;

  let fetchOptions: RequestInit = {
    ...init,
    headers: {
      "Content-Type": "application/json",
      ...init.headers,
    },
  };

  // Server-side: optional custom dispatcher for self-signed TLS (Node 24+ uses undici for native fetch).
  // Node 24's global fetch accepts the same `dispatcher` option; undici is bundled in Node.
  if (typeof window === "undefined" && allowSelfSigned && url.startsWith("https://")) {
    try {
      const { Agent } = await import("undici");
      const dispatcher = new Agent({
        connect: { rejectUnauthorized: false },
      });
      (fetchOptions as RequestInit & { dispatcher?: unknown }).dispatcher = dispatcher;
    } catch {
      // undici not available or import failed; proceed with default fetch
    }
  }

  const res = await fetch(url, fetchOptions);
  if (!res.ok) {
    const text = await res.text();
    let body: unknown;
    try {
      body = JSON.parse(text);
    } catch {
      body = { message: text };
    }
    throw new ApiError(res.status, res.statusText, body);
  }

  const contentType = res.headers.get("content-type");
  if (contentType?.includes("application/json")) {
    return res.json() as Promise<T>;
  }
  return res.text() as Promise<T>;
}

export class ApiError extends Error {
  constructor(
    public status: number,
    statusText: string,
    public body: unknown
  ) {
    super(`API ${status}: ${statusText}`);
    this.name = "ApiError";
  }
}

/** Base URL for the certificate-service API (for use in API functions). */
export function getCertificateServiceBaseUrl(): string {
  return getBaseUrl();
}
