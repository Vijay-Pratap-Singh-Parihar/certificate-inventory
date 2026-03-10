"use client";

import { useCallback, useMemo, useRef, useState } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { ChevronDown, ChevronLeft, ChevronRight, ChevronUp } from "lucide-react";
import type { CertificateStatus } from "@/types/certificate";

const ROW_HEIGHT = 52;
const VIRTUALIZE_THRESHOLD = 50;

export type DataGridRow = {
  id: string;
  /** Display name (subject or domain). */
  domainOrName: string;
  /** Certificate type (e.g. TLS). */
  type: string;
  issuer: string;
  status: CertificateStatus;
  expiresOn: string;
  lastUpdated: string;
  /** Optional: highlight row (e.g. expiring soon) */
  highlight?: boolean;
};

export type SortKey = "id" | "domainOrName" | "type" | "issuer" | "status" | "expiresOn" | "lastUpdated";
export type SortOrder = "asc" | "desc";

type DataGridProps = {
  rows: DataGridRow[];
  searchPlaceholder?: string;
  searchValue: string;
  onSearchChange: (value: string) => void;
  statusFilter: string;
  onStatusFilterChange: (value: string) => void;
  /** Pagination */
  page: number;
  totalPages: number;
  total: number;
  limit: number;
  onPageChange: (page: number) => void;
  onRowClick: (row: DataGridRow) => void;
  /** Optional: loading state for skeleton */
  isLoading?: boolean;
};

const STATUS_OPTIONS = ["All Statuses", "Valid", "Expiring Soon", "Expired"];

const COLS: { key: SortKey; label: string; className?: string }[] = [
  { key: "id", label: "ID", className: "w-20" },
  { key: "domainOrName", label: "Name" },
  { key: "type", label: "Type", className: "w-16" },
  { key: "issuer", label: "Issuer" },
  { key: "status", label: "Status", className: "w-28" },
  { key: "expiresOn", label: "Expires On", className: "w-28" },
  { key: "lastUpdated", label: "Last Updated", className: "w-28" },
];

function StatusBadge({ status }: { status: CertificateStatus }) {
  const styles: Record<CertificateStatus, string> = {
    Valid: "bg-green-100 text-green-800",
    "Expiring Soon": "bg-orange-100 text-orange-800",
    Expired: "bg-red-100 text-red-800",
  };
  return (
    <span
      className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${styles[status]}`}
    >
      {status}
    </span>
  );
}

function sortRows(rows: DataGridRow[], sortKey: SortKey, order: SortOrder): DataGridRow[] {
  return [...rows].sort((a, b) => {
    const aVal = a[sortKey] ?? "";
    const bVal = b[sortKey] ?? "";
    const cmp = String(aVal).localeCompare(String(bVal), undefined, { numeric: true });
    return order === "asc" ? cmp : -cmp;
  });
}

export function DataGrid({
  rows,
  searchPlaceholder = "Search domains or issuers...",
  searchValue,
  onSearchChange,
  statusFilter,
  onStatusFilterChange,
  page,
  totalPages,
  total,
  limit,
  onPageChange,
  onRowClick,
  isLoading = false,
}: DataGridProps) {
  const [sortKey, setSortKey] = useState<SortKey>("expiresOn");
  const [sortOrder, setSortOrder] = useState<SortOrder>("asc");
  const parentRef = useRef<HTMLDivElement>(null);

  const sortedRows = useMemo(
    () => sortRows(rows, sortKey, sortOrder),
    [rows, sortKey, sortOrder]
  );

  const handleSort = useCallback((key: SortKey) => {
    setSortKey((prev) => {
      if (prev === key) {
        setSortOrder((o) => (o === "asc" ? "desc" : "asc"));
        return prev;
      }
      setSortOrder("asc");
      return key;
    });
  }, []);

  const start = total === 0 ? 0 : (page - 1) * limit + 1;
  const end = Math.min(page * limit, total);
  const useVirtual = sortedRows.length >= VIRTUALIZE_THRESHOLD;

  const rowVirtualizer = useVirtualizer({
    count: sortedRows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 10,
    enabled: useVirtual,
  });

  const virtualItems = rowVirtualizer.getVirtualItems();

  return (
    <div className="bg-white rounded-xl shadow-sm border border-gray-200 overflow-hidden flex flex-col">
      <div className="p-4 border-b border-gray-200 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div className="relative flex-1 max-w-md">
          <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
            <svg
              className="h-5 w-5 text-gray-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
              aria-hidden
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
              />
            </svg>
          </div>
          <input
            type="text"
            placeholder={searchPlaceholder}
            value={searchValue}
            onChange={(e) => onSearchChange(e.target.value)}
            className="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-lg focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
            aria-label="Search certificates by name or issuer"
          />
        </div>
        <div className="flex items-center gap-3">
          <select
            value={statusFilter}
            onChange={(e) => onStatusFilterChange(e.target.value)}
            className="block w-full pl-3 pr-10 py-2 text-base border border-gray-300 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm rounded-lg"
            aria-label="Filter by status"
          >
            {STATUS_OPTIONS.map((opt) => (
              <option key={opt} value={opt}>
                {opt}
              </option>
            ))}
          </select>
        </div>
      </div>

      <div
        ref={parentRef}
        className={`overflow-x-auto overflow-y-auto ${useVirtual ? "min-h-[400px]" : ""}`}
        role="region"
        aria-label="Certificate list"
      >
        <table className="min-w-full divide-y divide-gray-200" role="table">
          <thead className="bg-gray-50 text-left text-xs font-medium text-gray-500 uppercase tracking-wider sticky top-0 z-10">
            <tr role="row">
              {COLS.map(({ key, label, className }) => (
                <th
                  key={key}
                  scope="col"
                  role="columnheader"
                  className={`px-4 py-3 ${className ?? ""}`}
                >
                  <button
                    type="button"
                    onClick={() => handleSort(key)}
                    className="inline-flex items-center gap-1 hover:text-gray-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 rounded"
                    aria-label={`Sort by ${label} ${sortKey === key ? (sortOrder === "asc" ? "ascending" : "descending") : ""}`}
                  >
                    {label}
                    {sortKey === key ? (
                      sortOrder === "asc" ? (
                        <ChevronUp className="h-4 w-4" aria-hidden />
                      ) : (
                        <ChevronDown className="h-4 w-4" aria-hidden />
                      )
                    ) : null}
                  </button>
                </th>
              ))}
              <th scope="col" className="px-4 py-3 text-right w-20" role="columnheader">
                Action
              </th>
            </tr>
          </thead>
          <tbody className="bg-white divide-y divide-gray-200 text-sm" role="rowgroup">
            {isLoading ? (
              <TableSkeletonRows colSpan={COLS.length + 1} />
            ) : sortedRows.length === 0 ? (
              <tr role="row">
                <td colSpan={COLS.length + 1} className="px-6 py-8 text-center text-gray-500" role="cell">
                  No certificates found.
                </td>
              </tr>
            ) : useVirtual ? (
              <>
                {virtualItems[0] && virtualItems[0].start > 0 ? (
                  <tr aria-hidden>
                    <td colSpan={COLS.length + 1} style={{ height: virtualItems[0].start, padding: 0, border: "none", lineHeight: 0 }} />
                  </tr>
                ) : null}
                {virtualItems.map((virtualRow) => {
                  const row = sortedRows[virtualRow.index]!;
                  return (
                    <tr
                      key={row.id}
                      role="row"
                      tabIndex={0}
                      aria-label={`Certificate ${row.domainOrName}, ${row.status}`}
                      className={`transition-colors cursor-pointer ${row.highlight ? "bg-orange-50/30" : "hover:bg-gray-50"}`}
                      onClick={() => onRowClick(row)}
                      onKeyDown={(e) => {
                        if (e.key === "Enter" || e.key === " ") {
                          e.preventDefault();
                          onRowClick(row);
                        }
                      }}
                    >
                      <td className="px-4 py-3 whitespace-nowrap text-gray-600 font-mono text-xs" role="cell">
                        {row.id.slice(0, 8)}…
                      </td>
                      <td className="px-4 py-3 whitespace-nowrap font-medium text-gray-900" role="cell">
                        {row.domainOrName}
                      </td>
                      <td className="px-4 py-3 whitespace-nowrap text-gray-500" role="cell">
                        {row.type}
                      </td>
                      <td className="px-4 py-3 whitespace-nowrap text-gray-500" role="cell">
                        {row.issuer}
                      </td>
                      <td className="px-4 py-3 whitespace-nowrap" role="cell" suppressHydrationWarning>
                        <StatusBadge status={row.status} />
                      </td>
                      <td
                        className={`px-4 py-3 whitespace-nowrap ${row.status === "Expiring Soon" ? "text-orange-600 font-medium" : "text-gray-500"}`}
                        role="cell"
                        suppressHydrationWarning
                      >
                        {row.expiresOn}
                      </td>
                      <td className="px-4 py-3 whitespace-nowrap text-gray-500" role="cell">
                        {row.lastUpdated}
                      </td>
                      <td className="px-4 py-3 whitespace-nowrap text-right text-sm font-medium" role="cell">
                        <button
                          type="button"
                          onClick={(e) => {
                            e.stopPropagation();
                            onRowClick(row);
                          }}
                          className="text-indigo-600 hover:text-indigo-900 focus:outline-none focus:ring-2 focus:ring-indigo-500 rounded"
                          aria-label={`View ${row.domainOrName}`}
                        >
                          View
                        </button>
                      </td>
                    </tr>
                  );
                })}
                {virtualItems.length > 0 && (() => {
                  const last = virtualItems[virtualItems.length - 1]!;
                  const trailing = rowVirtualizer.getTotalSize() - last.end;
                  if (trailing > 0) {
                    return (
                      <tr aria-hidden>
                        <td colSpan={COLS.length + 1} style={{ height: trailing, padding: 0, border: "none", lineHeight: 0 }} />
                      </tr>
                    );
                  }
                  return null;
                })()}
              </>
            ) : (
              sortedRows.map((row) => (
                <tr
                  key={row.id}
                  role="row"
                  tabIndex={0}
                  aria-label={`Certificate ${row.domainOrName}, ${row.status}`}
                  className={`transition-colors cursor-pointer ${row.highlight ? "bg-orange-50/30" : "hover:bg-gray-50"}`}
                  onClick={() => onRowClick(row)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" || e.key === " ") {
                      e.preventDefault();
                      onRowClick(row);
                    }
                  }}
                >
                  <td className="px-4 py-3 whitespace-nowrap text-gray-600 font-mono text-xs" role="cell">
                    {row.id.length > 12 ? `${row.id.slice(0, 8)}…` : row.id}
                  </td>
                  <td className="px-4 py-3 whitespace-nowrap font-medium text-gray-900" role="cell">
                    {row.domainOrName}
                  </td>
                  <td className="px-4 py-3 whitespace-nowrap text-gray-500" role="cell">
                    {row.type}
                  </td>
                  <td className="px-4 py-3 whitespace-nowrap text-gray-500" role="cell">
                    {row.issuer}
                  </td>
                  <td className="px-4 py-3 whitespace-nowrap" role="cell" suppressHydrationWarning>
                    <StatusBadge status={row.status} />
                  </td>
                  <td
                    className={`px-4 py-3 whitespace-nowrap ${row.status === "Expiring Soon" ? "text-orange-600 font-medium" : "text-gray-500"}`}
                    role="cell"
                    suppressHydrationWarning
                  >
                    {row.expiresOn}
                  </td>
                  <td className="px-4 py-3 whitespace-nowrap text-gray-500" role="cell">
                    {row.lastUpdated}
                  </td>
                  <td className="px-4 py-3 whitespace-nowrap text-right text-sm font-medium" role="cell">
                    <button
                      type="button"
                      onClick={(e) => {
                        e.stopPropagation();
                        onRowClick(row);
                      }}
                      className="text-indigo-600 hover:text-indigo-900"
                      aria-label={`View ${row.domainOrName}`}
                    >
                      View
                    </button>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      <div className="px-4 py-3 border-t border-gray-200 flex items-center justify-between sm:px-6 bg-gray-50">
        <div className="flex-1 flex justify-between sm:hidden">
          <button
            type="button"
            disabled={page <= 1}
            onClick={() => onPageChange(page - 1)}
            className="relative inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
            aria-label="Previous page"
          >
            Previous
          </button>
          <button
            type="button"
            disabled={page >= totalPages}
            onClick={() => onPageChange(page + 1)}
            className="ml-3 relative inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
            aria-label="Next page"
          >
            Next
          </button>
        </div>
        <div className="hidden sm:flex-1 sm:flex sm:items-center sm:justify-between">
          <div>
            <p className="text-sm text-gray-700">
              Showing <span className="font-medium">{start}</span> to{" "}
              <span className="font-medium">{end}</span> of{" "}
              <span className="font-medium">{total}</span> results
            </p>
          </div>
          <nav
            className="relative z-0 inline-flex rounded-md shadow-sm -space-x-px"
            aria-label="Pagination"
          >
            <button
              type="button"
              disabled={page <= 1}
              onClick={() => onPageChange(page - 1)}
              className="relative inline-flex items-center px-2 py-2 rounded-l-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
              aria-label="Previous page"
            >
              <ChevronLeft className="h-5 w-5" />
            </button>
            {Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
              let p: number;
              if (totalPages <= 5) p = i + 1;
              else if (page <= 3) p = i + 1;
              else if (page >= totalPages - 2) p = totalPages - 4 + i;
              else p = page - 2 + i;
              return (
                <button
                  key={p}
                  type="button"
                  onClick={() => onPageChange(p)}
                  className={`relative inline-flex items-center px-4 py-2 border text-sm font-medium ${
                    p === page
                      ? "border-gray-300 bg-indigo-50 text-indigo-600 z-10"
                      : "border-gray-300 bg-white text-gray-700 hover:bg-gray-50"
                  }`}
                  aria-label={p === page ? `Page ${p} (current)` : `Page ${p}`}
                >
                  {p}
                </button>
              );
            })}
            <button
              type="button"
              disabled={page >= totalPages}
              onClick={() => onPageChange(page + 1)}
              className="relative inline-flex items-center px-2 py-2 rounded-r-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
              aria-label="Next page"
            >
              <ChevronRight className="h-5 w-5" />
            </button>
          </nav>
        </div>
      </div>
    </div>
  );
}

function TableSkeletonRows({ colSpan }: { colSpan: number }) {
  return (
    <>
      {Array.from({ length: 8 }, (_, i) => (
        <tr key={i} role="row">
          {Array.from({ length: colSpan }, (_, j) => (
            <td key={j} className="px-4 py-3" role="cell">
              <div className="h-4 bg-gray-200 rounded animate-pulse" />
            </td>
          ))}
        </tr>
      ))}
    </>
  );
}
