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

const DEFAULT_BASE = "https://certificate-inventory.onrender.com";

/** True if the value is a valid absolute API base URL (avoids relative URLs on GitHub Pages). */
function isAbsoluteBaseUrl(value: string | undefined): boolean {
  return Boolean(value && (value.startsWith("http://") || value.startsWith("https://")));
}

/** Resolved API base URL. Never returns a relative path so requests always hit the backend on static export (e.g. GitHub Pages). */
function getBaseUrl(): string {
  if (typeof window !== "undefined") {
    const base = process.env.NEXT_PUBLIC_CERTIFICATE_SERVICE_URL ?? DEFAULT_BASE;
    return isAbsoluteBaseUrl(base) ? base : DEFAULT_BASE;
  }
  const base =
    process.env.CERTIFICATE_SERVICE_URL ||
    process.env.NEXT_PUBLIC_CERTIFICATE_SERVICE_URL ||
    DEFAULT_BASE;
  return isAbsoluteBaseUrl(base) ? base : DEFAULT_BASE;
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
  // Ensure we never build a relative URL (e.g. when basePath is set on GitHub Pages).
  const baseUrl = base.replace(/\/$/, "");
  const pathPart = path.startsWith("/") ? path : `/${path}`;
  const url = path.startsWith("http") ? path : `${baseUrl}${pathPart}`;

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
