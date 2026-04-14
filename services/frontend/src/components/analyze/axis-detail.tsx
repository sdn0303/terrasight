"use client";

import type { TlsResponse } from "@/lib/api/schemas/score";

type AxisKey = keyof TlsResponse["axes"];

interface AxisDetailProps {
  axisKey: AxisKey;
  axis: TlsResponse["axes"][AxisKey];
}

export function AxisDetail({ axis }: AxisDetailProps) {
  return (
    <div className="ml-16 mr-2 mb-2 rounded-lg p-2 space-y-1 bg-ds-bg-tertiary/50">
      {axis.sub.map((sub) => (
        <div key={sub.id} className="flex items-center gap-2">
          <span className="w-20 text-[9px] text-ds-text-muted truncate">
            {sub.id}
          </span>
          <div className="flex-1 h-1.5 rounded-full overflow-hidden bg-ds-bg-secondary">
            <div
              className="h-full rounded-full"
              style={{
                width: `${sub.score}%`,
                background: sub.available ? "#a3a3a3" : "#525252",
                opacity: sub.available ? 1 : 0.3,
              }}
            />
          </div>
          <span
            className={`w-6 text-right text-[9px] font-mono ${sub.available ? "text-ds-text-heading" : "text-ds-text-muted"}`}
          >
            {Math.round(sub.score)}
          </span>
        </div>
      ))}
      <div className="text-[9px] text-ds-text-muted mt-1 pt-1 border-t border-ds-border-primary">
        confidence: {Math.round(axis.confidence * 100)}% | weight:{" "}
        {Math.round(axis.weight * 100)}%
      </div>
    </div>
  );
}
