"use client";

import { useState } from "react";
import { useShallow } from "zustand/react/shallow";
import { Skeleton } from "@/components/ui/skeleton";
import { useMediaQuery } from "@/hooks/use-media-query";
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
  const bbox = useMapStore(
    useShallow((s) => {
      const { latitude, longitude, zoom } = s.viewState;
      const latRange = 180 / 2 ** zoom;
      const lngRange = 360 / 2 ** zoom;
      return {
        south: latitude - latRange / 2,
        west: longitude - lngRange / 2,
        north: latitude + latRange / 2,
        east: longitude + lngRange / 2,
      };
    }),
  );
  const { data: stats, isLoading } = useStats(bbox);
  const isTablet = useMediaQuery("(min-width: 768px)");
  const isDesktop = useMediaQuery("(min-width: 1280px)");
  const isMobile = !isTablet;

  // Mobile: hidden by default, toggled by a floating button
  const [mobileVisible, setMobileVisible] = useState(false);

  if (isMobile) {
    return (
      <>
        {/* Floating tap-to-show button on mobile */}
        <button
          type="button"
          onClick={() => setMobileVisible((v) => !v)}
          className="fixed bottom-10 right-4 z-40 rounded px-3 py-1.5 text-[10px] tracking-[0.1em]"
          style={{
            background: "var(--bg-secondary)",
            border: "1px solid var(--border-primary)",
            color: "var(--accent-cyan)",
          }}
          aria-expanded={mobileVisible}
          aria-controls="dashboard-stats-panel"
          aria-label={mobileVisible ? "Hide area statistics" : "Show area statistics"}
        >
          {mobileVisible ? "HIDE STATS" : "SHOW STATS"}
        </button>

        {mobileVisible && (
          <div
            id="dashboard-stats-panel"
            className="fixed left-0 right-0 grid grid-cols-2 gap-2 px-3 py-2"
            style={{
              bottom: 28,
              background: "var(--bg-secondary)",
              borderTop: "1px solid var(--border-primary)",
              zIndex: 30,
            }}
            aria-label="Area statistics"
            role="region"
          >
            {isLoading ? (
              Array.from({ length: 4 }).map((_, i) => (
                <div
                  key={i}
                  className="rounded-lg p-3"
                  style={{ background: "var(--bg-tertiary)" }}
                >
                  <Skeleton className="h-3 w-16 mb-2" />
                  <Skeleton className="h-6 w-24" />
                </div>
              ))
            ) : stats ? (
              <>
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
                  value={`${Math.round(stats.risk.composite_risk * 100)}%`}
                  color={
                    Math.round(stats.risk.composite_risk * 100) > 30
                      ? "var(--accent-danger)"
                      : "var(--accent-success)"
                  }
                />
                <StatCard
                  label="FACILITIES"
                  value={String(stats.facilities.schools + stats.facilities.medical)}
                  sub={`${stats.facilities.schools} schools, ${stats.facilities.medical} medical`}
                />
              </>
            ) : null}
          </div>
        )}
      </>
    );
  }

  // Tablet: 80px height, 2x2 grid
  // Desktop: 120px height, single row
  const panelHeight = isDesktop ? 120 : 80;
  const gridClass = isDesktop ? "flex gap-3" : "grid grid-cols-2 gap-2";

  if (isLoading) {
    return (
      <div
        className={`fixed left-0 right-0 px-4 py-3 ${gridClass}`}
        style={{
          bottom: 28,
          height: panelHeight,
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

  const riskPct = Math.round(stats.risk.composite_risk * 100);

  return (
    <div
      className={`fixed left-0 right-0 px-4 py-3 ${gridClass}`}
      style={{
        bottom: 28,
        height: panelHeight,
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
