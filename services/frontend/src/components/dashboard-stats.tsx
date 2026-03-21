"use client";

import { Skeleton } from "@/components/ui/skeleton";
import { useStats } from "@/features/stats/api/use-stats";
import { useMapStore } from "@/stores/map-store";

function StatCard({
  label,
  value,
  sub,
  color,
}: {
  label: string;
  value: string;
  sub?: string;
  color?: string;
}) {
  return (
    <div
      className="flex-1 rounded-lg p-3"
      style={{ background: "var(--bg-tertiary)" }}
    >
      <div
        className="text-[9px] tracking-[0.15em] mb-1"
        style={{ color: "var(--text-muted)" }}
      >
        {label}
      </div>
      <div
        className="text-lg font-bold"
        style={{ color: color ?? "var(--accent-cyan)" }}
      >
        {value}
      </div>
      {sub && (
        <div className="text-[10px]" style={{ color: "var(--text-secondary)" }}>
          {sub}
        </div>
      )}
    </div>
  );
}

export function DashboardStats() {
  const bbox = useMapStore((s) => s.getBBox());
  const { data: stats, isLoading } = useStats(bbox);

  if (isLoading) {
    return (
      <div
        className="fixed left-0 right-0 flex gap-3 px-4 py-3"
        style={{
          bottom: 28,
          height: 120,
          background: "var(--bg-secondary)",
          borderTop: "1px solid var(--border-primary)",
          zIndex: 30,
        }}
        aria-label="Area statistics loading"
        aria-busy="true"
      >
        {Array.from({ length: 4 }).map((_, i) => (
          <div
            key={i}
            className="flex-1 rounded-lg p-3"
            style={{ background: "var(--bg-tertiary)" }}
          >
            <Skeleton className="h-3 w-16 mb-2" />
            <Skeleton className="h-6 w-24" />
          </div>
        ))}
      </div>
    );
  }

  if (!stats) return null;

  const riskPct = Math.round(stats.risk.avg_composite_risk * 100);

  return (
    <div
      className="fixed left-0 right-0 flex gap-3 px-4 py-3"
      style={{
        bottom: 28,
        height: 120,
        background: "var(--bg-secondary)",
        borderTop: "1px solid var(--border-primary)",
        zIndex: 30,
      }}
      aria-label="Area statistics"
      role="region"
    >
      <StatCard
        label="AVG PRICE"
        value={`¥${stats.land_price.avg_per_sqm.toLocaleString()}`}
        sub={`med: ¥${stats.land_price.median_per_sqm.toLocaleString()}`}
      />
      <StatCard
        label="LISTINGS"
        value={String(stats.land_price.count)}
      />
      <StatCard
        label="RISK"
        value={`${riskPct}%`}
        color={riskPct > 30 ? "var(--accent-danger)" : "var(--accent-success)"}
      />
      <StatCard
        label="FACILITIES"
        value={String(stats.facilities.schools + stats.facilities.medical)}
        sub={`${stats.facilities.schools} schools, ${stats.facilities.medical} medical`}
      />
    </div>
  );
}
