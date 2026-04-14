"use client";

import { useTranslations } from "next-intl";
import { useAreaStats } from "@/features/area-stats/api/use-area-stats";
import { useMapStore } from "@/stores/map-store";

export function AreaCard() {
  const t = useTranslations("explore.areaCard");
  const selectedArea = useMapStore((s) => s.selectedArea);
  const { data, isPending } = useAreaStats(selectedArea?.code ?? null);

  if (!selectedArea) return null;

  function formatAvgPrice(val: number | null | undefined): string {
    if (val == null) return "\u2014";
    return `\u00a5${val.toLocaleString()}/sqm`;
  }

  function formatRisk(val: number | undefined): string {
    if (val == null) return "\u2014";
    return `${(val * 100).toFixed(1)}%`;
  }

  function formatFacilities(
    schools: number | undefined,
    medical: number | undefined,
  ): string {
    if (schools == null || medical == null) return "\u2014";
    return String(schools + medical);
  }

  const skeletonCls = "h-4 w-12 rounded bg-ds-bg-tertiary animate-pulse";

  const stats = [
    {
      label: t("population"),
      value: isPending ? <span className={skeletonCls} /> : "\u2014",
    },
    {
      label: t("avgPrice"),
      value: isPending ? (
        <span className={skeletonCls} />
      ) : (
        formatAvgPrice(data?.land_price.avg_per_sqm)
      ),
    },
    {
      label: t("risk"),
      value: isPending ? (
        <span className={skeletonCls} />
      ) : (
        formatRisk(data?.risk.composite_risk)
      ),
    },
    {
      label: t("avgTls"),
      value: isPending ? (
        <span className={skeletonCls} />
      ) : (
        formatFacilities(data?.facilities.schools, data?.facilities.medical)
      ),
    },
  ];

  return (
    <div className="px-4 py-3">
      <div className="rounded-lg p-3 bg-ds-bg-tertiary/50 border border-[rgba(99,102,241,0.08)]">
        <div className="text-sm font-medium text-ds-text-heading mb-2">
          {selectedArea.name}
        </div>
        <div className="grid grid-cols-2 gap-2">
          {stats.map((stat) => (
            <div key={stat.label}>
              <div className="text-[10px] font-medium uppercase tracking-wider text-ds-text-muted">
                {stat.label}
              </div>
              <div className="text-sm font-bold text-ds-accent-primary">
                {stat.value}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
