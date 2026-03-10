import { AlertTriangle } from "lucide-react";

type MetricCardProps = {
  label: string;
  value: number | string;
  variant?: "default" | "warning";
};

export function MetricCard({ label, value, variant = "default" }: MetricCardProps) {
  if (variant === "warning") {
    return (
      <div className="bg-white rounded-xl shadow-sm border border-orange-200 p-6 flex flex-col relative overflow-hidden">
        <div className="absolute top-0 right-0 p-4 opacity-10 text-orange-600" aria-hidden>
          <AlertTriangle className="w-16 h-16" />
        </div>
        <dt className="text-sm font-medium text-orange-600 truncate">{label}</dt>
        <dd className="mt-2 text-3xl font-semibold text-gray-900" suppressHydrationWarning>{value}</dd>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6 flex flex-col">
      <dt className="text-sm font-medium text-gray-500 truncate">{label}</dt>
      <dd className="mt-2 text-3xl font-semibold text-gray-900" suppressHydrationWarning>{value}</dd>
    </div>
  );
}

/** Skeleton placeholder while metrics are loading. */
export function MetricCardSkeleton({ variant }: { variant?: "default" | "warning" }) {
  const borderClass = variant === "warning" ? "border-orange-200" : "border-gray-200";
  return (
    <div className={`bg-white rounded-xl shadow-sm border ${borderClass} p-6 flex flex-col animate-pulse`}>
      <div className="h-4 w-24 bg-gray-200 rounded" />
      <div className="mt-2 h-8 w-16 bg-gray-200 rounded" />
    </div>
  );
}
