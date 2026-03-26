"use client";

import { useTranslations } from "next-intl";
import { useMapStore } from "@/stores/map-store";

export function AreaCard() {
  const t = useTranslations("explore.areaCard");
  const selectedArea = useMapStore((s) => s.selectedArea);

  if (!selectedArea) return null;

  const stats = [
    { label: t("population"), value: "\u2014" },
    { label: t("avgPrice"), value: "\u2014" },
    { label: t("risk"), value: "\u2014" },
    { label: t("avgTls"), value: "\u2014" },
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
