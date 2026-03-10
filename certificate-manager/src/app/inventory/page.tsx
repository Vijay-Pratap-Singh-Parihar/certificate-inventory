import { getCertificates, getDashboardMetrics } from "@/lib/api";
import { InventoryView } from "./InventoryView";
import type { Certificate, PaginatedResponse, DashboardMetrics } from "@/types/certificate";

const LIMIT = 10;

const emptyData: PaginatedResponse<Certificate> = {
  items: [],
  total: 0,
  page: 1,
  limit: LIMIT,
  total_pages: 0,
};

const emptyMetrics: DashboardMetrics = { total: 0, expiring_soon: 0 };

export default async function InventoryPage() {
  let initialData = emptyData;
  let initialMetrics = emptyMetrics;
  try {
    const [data, metrics] = await Promise.all([
      getCertificates({ page: 1, limit: LIMIT }),
      getDashboardMetrics(),
    ]);
    initialData = data;
    initialMetrics = metrics;
  } catch {
    // Service unreachable or other error; render with empty data so the page still loads.
  }

  return (
    <InventoryView initialData={initialData} initialMetrics={initialMetrics} />
  );
}
