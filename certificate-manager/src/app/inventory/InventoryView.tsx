"use client";

import { useState, useMemo } from "react";
import useSWR from "swr";
import { AppShell } from "@/components/AppShell";
import { MetricCard, MetricCardSkeleton } from "@/components/MetricCard";
import { DataGrid, type DataGridRow } from "@/components/DataGrid";
import { CertificateModal } from "@/components/CertificateModal";
import type { Certificate, PaginatedResponse, DashboardMetrics } from "@/types/certificate";
import { getCertificateStatus } from "@/types/certificate";
import { getCertificates, getDashboardMetrics, createCertificate } from "@/lib/api";

function formatExpires(iso: string): string {
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

function formatLastUpdated(iso: string | null | undefined): string {
  if (!iso) return "—";
  try {
    return new Date(iso).toLocaleDateString("en-US", {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
  } catch {
    return "—";
  }
}

function certToRow(c: Certificate): DataGridRow {
  const status = getCertificateStatus(c.expiration);
  return {
    id: c.id,
    domainOrName: c.subject,
    type: "TLS",
    issuer: c.issuer,
    status,
    expiresOn: formatExpires(c.expiration),
    lastUpdated: formatLastUpdated(c.last_updated ?? c.expiration),
    highlight: status === "Expiring Soon",
  };
}

const LIMIT = 10;

export function InventoryView({
  initialData,
  initialMetrics,
}: {
  initialData: PaginatedResponse<Certificate>;
  initialMetrics: DashboardMetrics;
}) {
  const [page, setPage] = useState(1);
  const [statusFilter, setStatusFilter] = useState("All Statuses");
  const [search, setSearch] = useState("");
  const [modalCert, setModalCert] = useState<Certificate | null>(null);
  const [modalOpen, setModalOpen] = useState(false);
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);

  const listKey = `/certificates?page=${page}&limit=${LIMIT}&status=${statusFilter}`;
  const { data: listData = initialData, mutate: mutateList, isLoading: listLoading } = useSWR(listKey, () =>
    getCertificates({ page, limit: LIMIT, status: statusFilter })
  );

  const { data: metrics = initialMetrics, mutate: mutateMetrics, isLoading: metricsLoading } = useSWR(
    "/certificates/metrics",
    getDashboardMetrics
  );

  const rows = useMemo(() => {
    let items = listData.items;
    if (search.trim()) {
      const q = search.trim().toLowerCase();
      items = items.filter(
        (c) =>
          c.subject.toLowerCase().includes(q) || c.issuer.toLowerCase().includes(q)
      );
    }
    return items.map(certToRow);
  }, [listData.items, search]);

  const certificatesById = useMemo(() => {
    const m = new Map<string, Certificate>();
    listData.items.forEach((c) => m.set(c.id, c));
    return m;
  }, [listData.items]);

  const handleRowClick = (row: DataGridRow) => {
    const cert = certificatesById.get(row.id) ?? null;
    setModalCert(cert);
    setModalOpen(true);
  };

  return (
    <AppShell
      breadcrumb={
        <>
          Home <span className="mx-2">/</span>{" "}
          <span className="text-gray-900">Inventory</span>
        </>
      }
      action={
        <button
          type="button"
          onClick={() => setIsCreateModalOpen(true)}
          className="bg-indigo-600 hover:bg-indigo-700 text-white px-4 py-2 rounded-md text-sm font-medium shadow-sm transition-colors"
        >
          + New Certificate
        </button>
      }
    >
      <div className="flex-1 overflow-y-auto p-4 sm:p-6 lg:p-8">
        <div className="max-w-7xl mx-auto space-y-6">
          <div>
            <h1 className="text-2xl font-bold text-gray-900">Certificate Inventory</h1>
            <p className="text-sm text-gray-500 mt-1">
              Manage and monitor all active and expiring TLS/mTLS certificates.
            </p>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 lg:gap-6">
            {metricsLoading ? (
              <MetricCardSkeleton />
            ) : (
              <MetricCard label="Total Certificates" value={metrics.total.toLocaleString()} />
            )}
            {metricsLoading ? (
              <MetricCardSkeleton variant="warning" />
            ) : (
              <MetricCard
                label="Expiring Soon (Next 30 Days)"
                value={metrics.expiring_soon}
                variant="warning"
              />
            )}
          </div>

          <DataGrid
            rows={rows}
            searchValue={search}
            onSearchChange={setSearch}
            statusFilter={statusFilter}
            onStatusFilterChange={setStatusFilter}
            page={listData.page}
            totalPages={Math.max(1, listData.total_pages)}
            total={listData.total}
            limit={listData.limit}
            onPageChange={setPage}
            onRowClick={handleRowClick}
            isLoading={listLoading}
          />
        </div>
      </div>

      <CertificateModal
        certificate={modalCert}
        open={modalOpen}
        onClose={() => {
          setModalOpen(false);
          setModalCert(null);
        }}
      />
      <CertificateModal
        open={isCreateModalOpen}
        certificate={null}
        onClose={() => setIsCreateModalOpen(false)}
        mode="create"
        onCreateSubmit={async (pem) => {
          await createCertificate(pem);
        }}
        onSuccess={() => {
          setIsCreateModalOpen(false);
          void mutateList();
          void mutateMetrics();
        }}
      />
    </AppShell>
  );
}
