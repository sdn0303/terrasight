"use client";

import { useTranslations } from "next-intl";
import { useMapStore } from "@/stores/map-store";
import { useAreaStats } from "@/features/area-stats/api/use-area-stats";

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

  const skeletonCls = "h-4 w-12 rounded bg-neutral-700 animate-pulse";

  const stats = [
    {
      label: t("population"),
      value: isPending ? (
        <span className={skeletonCls} />
      ) : (
        "\u2014"
      ),
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
      <div className="rounded-lg p-3 bg-neutral-800/50">
        <div className="text-sm font-medium text-neutral-200 mb-2">
          {selectedArea.name}
        </div>
        <div className="grid grid-cols-2 gap-2">
          {stats.map((stat) => (
            <div key={stat.label}>
              <div className="text-[9px] font-mono tracking-wider text-neutral-500">
                {stat.label}
              </div>
              <div className="text-sm font-bold text-cyan-400">
                {stat.value}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
