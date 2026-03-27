"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";
import type { TlsResponse } from "@/lib/schemas";
import { AxisDetail } from "./axis-detail";

const AXIS_KEYS = [
  "disaster",
  "terrain",
  "livability",
  "future",
  "price",
] as const;
type AxisKey = (typeof AXIS_KEYS)[number];

const AXIS_COLORS: Record<AxisKey, string> = {
  disaster: "#ef4444",
  terrain: "#f59e0b",
  livability: "#14b8a6",
  future: "#3b82f6",
  price: "#10b981",
};

interface AxisBarListProps {
  axes: TlsResponse["axes"];
}

export function AxisBarList({ axes }: AxisBarListProps) {
  const t = useTranslations("axis");
  const [expanded, setExpanded] = useState<AxisKey | null>(null);

  return (
    <div className="px-4 space-y-1">
      {AXIS_KEYS.map((key) => {
        const axis = axes[key];
        const color = AXIS_COLORS[key];
        const isExpanded = expanded === key;
        return (
          <div key={key}>
            <button
              type="button"
              onClick={() => setExpanded(isExpanded ? null : key)}
              className="flex items-center gap-2 w-full py-1.5 text-left"
            >
              <span className="w-14 text-[10px] truncate" style={{ color }}>
                {t(key)}
              </span>
              <div className="flex-1 h-2 rounded-full overflow-hidden bg-ds-bg-tertiary">
                <div
                  className="h-full rounded-full transition-all"
                  style={{ width: `${axis.score}%`, background: color }}
                />
              </div>
              <span className="w-8 text-right text-[11px] font-mono text-ds-text-heading">
                {Math.round(axis.score)}
              </span>
            </button>
            {isExpanded && <AxisDetail axisKey={key} axis={axis} />}
          </div>
        );
      })}
    </div>
  );
}
