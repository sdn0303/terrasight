"use client";

import type { TlsResponse } from "@/lib/schemas";

const PATTERNS = [
  { key: "value_discovery" as const, label: "Value Discovery", desc: "Safe but undervalued" },
  { key: "demand_signal" as const, label: "Demand Signal", desc: "Convenient + growing" },
  { key: "ground_safety" as const, label: "Ground Safety", desc: "Disaster x terrain" },
];

interface CrossAnalysisProps {
  crossAnalysis: TlsResponse["cross_analysis"];
}

export function CrossAnalysis({ crossAnalysis }: CrossAnalysisProps) {
  return (
    <div className="px-4 py-2">
      <div className="text-[9px] font-mono tracking-wider text-neutral-500 mb-2">CROSS ANALYSIS</div>
      <div className="space-y-1.5">
        {PATTERNS.map(({ key, label }) => {
          const value = crossAnalysis[key];
          return (
            <div key={key} className="flex items-center gap-2">
              <span className="w-28 text-[10px] text-neutral-400">{label}</span>
              <div className="flex-1 h-1.5 rounded-full overflow-hidden bg-neutral-800">
                <div className="h-full rounded-full bg-cyan-500" style={{ width: `${value}%` }} />
              </div>
              <span className="w-6 text-right text-[10px] font-mono text-neutral-200">{Math.round(value)}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
