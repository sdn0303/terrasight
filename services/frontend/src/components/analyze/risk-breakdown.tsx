"use client";

import type { TlsResponse } from "@/lib/api/schemas/score";

const RISK_COLORS: Record<string, string> = {
  flood: "#3b82f6",
  liquefaction: "#f59e0b",
  seismic: "#ef4444",
  tsunami: "#06b6d4",
  landslide: "#84cc16",
};

interface RiskBreakdownProps {
  disasterAxis: TlsResponse["axes"]["disaster"];
}

export function RiskBreakdown({ disasterAxis }: RiskBreakdownProps) {
  return (
    <div className="px-4 py-2">
      <div className="text-[9px] font-semibold tracking-wider uppercase text-ds-text-muted mb-2">
        RISK BREAKDOWN
      </div>
      <div className="space-y-1.5">
        {disasterAxis.sub.map((sub) => (
          <div key={sub.id} className="flex items-center gap-2">
            <div className="w-16 text-[9px] text-ds-text-secondary capitalize">
              {sub.id}
            </div>
            <div className="flex-1 h-2 bg-ds-bg-tertiary rounded-full overflow-hidden">
              <div
                className="h-full rounded-full transition-all"
                style={{
                  width: `${sub.score}%`,
                  backgroundColor: RISK_COLORS[sub.id] ?? "#818cf8",
                }}
              />
            </div>
            <div className="w-8 text-right text-[9px] text-ds-text-muted">
              {sub.available ? Math.round(sub.score) : "—"}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
