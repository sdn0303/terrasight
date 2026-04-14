"use client";

import { scoreGradient } from "@/lib/theme-tokens";

interface AxisBreakdownProps {
  axes: {
    disaster: number;
    terrain: number;
    livability: number;
    future: number;
    price: number;
  };
}

const AXIS_ROWS: { key: keyof AxisBreakdownProps["axes"]; label: string }[] = [
  { key: "disaster", label: "災害" },
  { key: "terrain", label: "地形" },
  { key: "livability", label: "生活" },
  { key: "future", label: "将来" },
  { key: "price", label: "価格" },
];

export function AxisBreakdown({ axes }: AxisBreakdownProps) {
  return (
    <div>
      <div
        className="mb-2 text-[8px] font-extrabold uppercase"
        style={{
          color: "var(--neutral-400)",
          letterSpacing: "0.7px",
        }}
      >
        5-Axis Breakdown
      </div>
      <div className="flex flex-col gap-1.5">
        {AXIS_ROWS.map(({ key, label }) => (
          <AxisRow key={key} label={label} value={axes[key]} />
        ))}
      </div>
    </div>
  );
}

function AxisRow({ label, value }: { label: string; value: number }) {
  const color =
    value >= 60
      ? "var(--score-good-start)"
      : value >= 40
        ? "var(--score-mid-start)"
        : "var(--score-bad-start)";
  return (
    <div className="flex items-center gap-2">
      <div
        className="w-[30px] text-[9px] font-semibold"
        style={{ color: "var(--neutral-600)" }}
      >
        {label}
      </div>
      <div
        className="relative h-1.5 flex-1 overflow-hidden"
        style={{
          background: "var(--neutral-100)",
          borderRadius: 999,
        }}
      >
        <div
          className="absolute inset-y-0 left-0"
          style={{
            width: `${Math.max(0, Math.min(100, value))}%`,
            background: scoreGradient(value),
            borderRadius: 999,
          }}
        />
      </div>
      <div
        className="w-[18px] text-right text-[9px] font-extrabold"
        style={{ color }}
      >
        {value}
      </div>
    </div>
  );
}
