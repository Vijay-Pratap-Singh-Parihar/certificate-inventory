"use client";

import { useState, useEffect } from "react";
import { X } from "lucide-react";
import type { Certificate } from "@/types/certificate";
import { getCertificateStatus } from "@/types/certificate";
import { getCertificatePem } from "@/lib/api";
import { ApiError } from "@/lib/fetcher";

type CertificateModalProps = {
  certificate: Certificate | null;
  open: boolean;
  onClose: () => void;
  /** When "create", shows upload form instead of certificate details. Default "view". */
  mode?: "view" | "create";
  /** Called when user submits the create form with PEM. On success, parent should close and refetch. */
  onCreateSubmit?: (pem: string) => Promise<void>;
  /** Called after create succeeds (after onCreateSubmit resolves). */
  onSuccess?: () => void;
};

function formatDate(iso: string): string {
  try {
    return new Date(iso).toLocaleDateString("en-US", {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
  } catch {
    return iso;
  }
}

/** Format certificate validity date in UTC so displayed date matches the certificate (avoids timezone off-by-one). */
function formatDateUTC(iso: string): string {
  try {
    return new Date(iso).toLocaleDateString("en-US", {
      year: "numeric",
      month: "short",
      day: "numeric",
      timeZone: "UTC",
    });
  } catch {
    return iso;
  }
}

function getErrorMessage(e: unknown): string {
  if (e instanceof ApiError && e.body && typeof e.body === "object" && "error" in e.body) {
    return String((e.body as { error?: unknown }).error);
  }
  if (e instanceof Error) return e.message;
  return String(e);
}

/** Sanitize certificate subject for use as filename (e.g. example.com.pem). */
function sanitizeFilename(subject: string): string {
  const s = subject.replace(/[^a-zA-Z0-9.-]/g, "_").trim() || "certificate";
  return s.slice(0, 196) + ".pem";
}

/** Allowed PEM block types for validation (certificate or key). */
const PEM_BLOCK_TYPES = [
  "CERTIFICATE",
  "RSA PRIVATE KEY",
  "PRIVATE KEY",
  "EC PRIVATE KEY",
  "CERTIFICATE REQUEST",
] as const;

/**
 * Returns true if the content contains at least one valid PEM begin/end block
 * (any of certificate or key types). Does not reject private keys.
 */
function isValidPemStructure(content: string): boolean {
  const trimmed = content.trim();
  if (!trimmed) return false;
  for (const label of PEM_BLOCK_TYPES) {
    const begin = `-----BEGIN ${label}-----`;
    const end = `-----END ${label}-----`;
    const beginIdx = trimmed.indexOf(begin);
    const endIdx = trimmed.indexOf(end, beginIdx);
    if (beginIdx !== -1 && endIdx !== -1 && endIdx > beginIdx) {
      const between = trimmed.slice(beginIdx + begin.length, endIdx).trim();
      if (between.length > 0 && /^[A-Za-z0-9+/=\s]+$/.test(between)) {
        return true;
      }
    }
  }
  return false;
}

export function CertificateModal({
  certificate,
  open,
  onClose,
  mode = "view",
  onCreateSubmit,
  onSuccess,
}: CertificateModalProps) {
  const [createPem, setCreatePem] = useState("");
  const [createAlias, setCreateAlias] = useState("");
  const [createError, setCreateError] = useState<string | null>(null);
  const [createSubmitting, setCreateSubmitting] = useState(false);
  const [uploadedFileName, setUploadedFileName] = useState<string | null>(null);
  const [downloadError, setDownloadError] = useState<string | null>(null);
  const [downloadLoading, setDownloadLoading] = useState(false);

  useEffect(() => {
    if (open && mode === "create") {
      setCreateError(null);
    }
  }, [open, mode]);

  const handleDownloadPem = async () => {
    if (!certificate) return;
    setDownloadError(null);
    setDownloadLoading(true);
    try {
      const { pem } = await getCertificatePem(certificate.id);
      const blob = new Blob([pem], { type: "application/x-pem-file" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = sanitizeFilename(certificate.subject);
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      console.error("Download .pem failed:", e);
      setDownloadError(getErrorMessage(e));
    } finally {
      setDownloadLoading(false);
    }
  };

  if (!open) return null;

  const isCreateMode = mode === "create";

  const status = certificate ? getCertificateStatus(certificate.expiration) : null;
  // Use API valid_from when present (parsed from PEM); fallback to legacy derivation for backward compatibility.
  const validFrom = certificate
    ? certificate.valid_from
      ? formatDateUTC(certificate.valid_from)
      : (() => {
          try {
            const d = new Date(certificate.expiration);
            d.setFullYear(d.getFullYear() - 1);
            return formatDateUTC(d.toISOString());
          } catch {
            return "—";
          }
        })()
    : "—";
  const validTo = certificate ? formatDateUTC(certificate.expiration) : "—";
  // Use API signature_algorithm when present (parsed from PEM); fallback to placeholder.
  const signatureAlgorithm = certificate?.signature_algorithm ?? "RSA-2048 / sha256WithRSAEncryption";

  return (
    <div
      id="detail-modal"
      className="fixed inset-0 overflow-hidden z-40"
      aria-labelledby="slide-over-title"
      role="dialog"
      aria-modal="true"
    >
      <div className="absolute inset-0 overflow-hidden">
        <div
          className="absolute inset-0 bg-gray-500 bg-opacity-75 transition-opacity"
          onClick={onClose}
          onKeyDown={(e) => e.key === "Escape" && onClose()}
          role="button"
          tabIndex={0}
          aria-label="Close panel"
        />
        <div className="pointer-events-none fixed inset-y-0 right-0 flex max-w-full pl-10 sm:pl-16">
          <div className="pointer-events-auto w-screen max-w-md">
            <div className="flex h-full flex-col overflow-y-scroll bg-white shadow-xl">
              <div className="bg-indigo-700 px-4 py-6 sm:px-6">
                <div className="flex items-center justify-between">
                  <h2 className="text-lg font-medium text-white" id="slide-over-title">
                    {isCreateMode ? "Add Certificate" : "Certificate Details"}
                  </h2>
                  <div className="ml-3 flex h-7 items-center">
                    <button
                      type="button"
                      onClick={onClose}
                      className="rounded-md bg-indigo-700 text-indigo-200 hover:text-white focus:outline-none"
                      aria-label="Close panel"
                    >
                      <X className="h-6 w-6" />
                    </button>
                  </div>
                </div>
                <div className="mt-1">
                  <p className="text-sm text-indigo-300" id="modal-domain">
                    {isCreateMode
                      ? "Paste or upload a PEM certificate"
                      : certificate
                        ? `${certificate.subject}${status ? ` • ${status}` : ""}`
                        : "—"}
                  </p>
                </div>
              </div>
              {isCreateMode ? (
                <form
                  className="relative flex-1 px-4 py-6 sm:px-6 space-y-6 flex flex-col"
                  onSubmit={async (e) => {
                    e.preventDefault();
                    const pem = createPem.trim();
                    if (!pem) {
                      setCreateError("Certificate PEM is required.");
                      return;
                    }
                    if (!isValidPemStructure(pem)) {
                      setCreateError(
                        "Invalid PEM format. Please upload a valid PEM encoded certificate or key."
                      );
                      return;
                    }
                    if (!onCreateSubmit || !onSuccess) return;
                    setCreateError(null);
                    setCreateSubmitting(true);
                    try {
                      await onCreateSubmit(pem);
                      onSuccess();
                      setCreatePem("");
                      setCreateAlias("");
                      setUploadedFileName(null);
                    } catch (err) {
                      const msg = getErrorMessage(err);
                      setCreateError(
                        /invalid pem|single x\.?509|x509 certificate/i.test(msg)
                          ? "Invalid PEM format. Please upload a valid PEM encoded certificate or key."
                          : msg
                      );
                    } finally {
                      setCreateSubmitting(false);
                    }
                  }}
                >
                  <div className="flex-1 space-y-4">
                    <div>
                      <label htmlFor="cert-pem" className="block text-sm font-medium text-gray-900">
                        Certificate PEM
                      </label>
                      <textarea
                        id="cert-pem"
                        rows={8}
                        className="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 text-sm shadow-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        placeholder={"-----BEGIN CERTIFICATE-----\n...\n-----END CERTIFICATE-----"}
                        value={createPem}
                        onChange={(e) => setCreatePem(e.target.value)}
                        disabled={createSubmitting}
                      />
                    </div>
                    <div>
                      <label htmlFor="cert-alias" className="block text-sm font-medium text-gray-900">
                        Optional name / alias
                      </label>
                      <input
                        id="cert-alias"
                        type="text"
                        className="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 text-sm shadow-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        placeholder="e.g. My API cert"
                        value={createAlias}
                        onChange={(e) => setCreateAlias(e.target.value)}
                        disabled={createSubmitting}
                      />
                    </div>
                    <div>
                      <input
                        id="cert-file"
                        type="file"
                        accept=".pem"
                        style={{ display: "none" }}
                        onChange={(e) => {
                          const file = e.target.files?.[0];
                          if (!file) return;
                          const name = file.name.toLowerCase();
                          if (!name.endsWith(".pem")) {
                            setUploadedFileName(null);
                            setCreateError("Please select a .pem file.");
                            return;
                          }
                          setCreateError(null);
                          setUploadedFileName(file.name);
                          const reader = new FileReader();
                          reader.onload = () => setCreatePem(String(reader.result ?? "").trim());
                          reader.readAsText(file);
                          e.target.value = "";
                        }}
                        disabled={createSubmitting}
                      />
                      <div className="mt-1 flex items-center">
                        <label
                          htmlFor="cert-file"
                          className="cursor-pointer rounded-md border-0 bg-indigo-50 px-4 py-2 text-sm font-medium text-indigo-700 hover:bg-indigo-100"
                        >
                          Choose File
                        </label>
                        {uploadedFileName && (
                          <span className="ml-2 text-sm text-gray-600">
                            {uploadedFileName}
                          </span>
                        )}
                      </div>
                    </div>
                    {createError && (
                      <p className="text-sm text-red-600" role="alert">
                        {createError}
                      </p>
                    )}
                  </div>
                  <div className="border-t border-gray-200 pt-4 flex justify-end gap-3">
                    <button
                      type="button"
                      onClick={onClose}
                      className="rounded-md bg-white px-3 py-2 text-sm font-semibold text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 hover:bg-gray-50"
                    >
                      Cancel
                    </button>
                    <button
                      type="submit"
                      disabled={createSubmitting}
                      className="rounded-md bg-indigo-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 disabled:opacity-50"
                    >
                      {createSubmitting ? "Adding…" : "Add Certificate"}
                    </button>
                  </div>
                </form>
              ) : (
                <>
                  <div className="relative flex-1 px-4 py-6 sm:px-6 space-y-6">
                    <div>
                      <h3 className="font-medium text-gray-900">Issuer</h3>
                      <p className="text-sm text-gray-500 mt-1">
                        {certificate?.issuer ?? "—"}
                      </p>
                    </div>
                    <div>
                      <h3 className="font-medium text-gray-900">Validity Period</h3>
                      <p className="text-sm text-gray-500 mt-1">
                        From: {validFrom}
                        <br />
                        To: {validTo}
                      </p>
                    </div>
                    <div>
                      <h3 className="font-medium text-gray-900">Cryptographic Algorithm</h3>
                      <p className="text-sm text-gray-500 mt-1">
                        {signatureAlgorithm}
                      </p>
                    </div>
                    <div>
                      <h3 className="font-medium text-gray-900">Subject Alternative Names</h3>
                      <ul className="text-sm text-gray-500 mt-1 list-disc list-inside">
                        {certificate?.san_entries?.length
                          ? certificate.san_entries.map((san) => (
                              <li key={san}>{san}</li>
                            ))
                          : ["—"]}
                      </ul>
                    </div>
                  </div>
                  <div className="border-t border-gray-200 px-4 py-4 sm:px-6 flex flex-col gap-3">
                    {downloadError && (
                      <p className="text-sm text-red-600" role="alert">
                        {downloadError}
                      </p>
                    )}
                    <div className="flex justify-end gap-3">
                      <button
                        type="button"
                        onClick={onClose}
                        className="rounded-md bg-white px-3 py-2 text-sm font-semibold text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 hover:bg-gray-50"
                      >
                        Close
                      </button>
                      <button
                        type="button"
                        onClick={handleDownloadPem}
                        disabled={downloadLoading}
                        className="rounded-md bg-indigo-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 disabled:opacity-50"
                      >
                        {downloadLoading ? "Downloading…" : "Download .pem"}
                      </button>
                    </div>
                  </div>
                </>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
