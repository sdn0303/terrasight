"use client";

import { SegmentedToggle } from "@/components/ui/segmented-toggle";
import { GRADIENT } from "@/lib/theme-tokens";
import { type RiskLevel, useFilterStore } from "@/stores/filter-store";

export function CriteriaSliders() {
  const tlsMin = useFilterStore((s) => s.criteria.tlsMin);
  const riskMax = useFilterStore((s) => s.criteria.riskMax);
  const priceRange = useFilterStore((s) => s.criteria.priceRange);
  const setCriteria = useFilterStore((s) => s.setCriteria);

  return (
    <div className="space-y-3.5">
      <TlsMinSlider
        value={tlsMin}
        onChange={(v) => setCriteria({ tlsMin: v })}
      />
      <RiskMaxToggle
        value={riskMax}
        onChange={(v) => setCriteria({ riskMax: v })}
      />
      <PriceRangeSlider
        value={priceRange}
        onChange={(v) => setCriteria({ priceRange: v })}
      />
    </div>
  );
}

function TlsMinSlider({
  value,
  onChange,
}: {
  value: number;
  onChange: (v: number) => void;
}) {
  return (
    <div>
      <div className="mb-1.5 flex items-center justify-between text-[10px]">
        <span style={{ color: "var(--neutral-600)" }}>TLS 最小スコア</span>
        <span
          className="font-extrabold"
          style={{ color: "var(--neutral-900)" }}
        >
          {value}+
        </span>
      </div>
      <input
        type="range"
        min={0}
        max={100}
        step={5}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
        aria-label="TLS minimum score"
        className="w-full"
      />
    </div>
  );
}

function RiskMaxToggle({
  value,
  onChange,
}: {
  value: RiskLevel;
  onChange: (v: RiskLevel) => void;
}) {
  return (
    <div>
      <div
        className="mb-1.5 text-[10px]"
        style={{ color: "var(--neutral-600)" }}
      >
        災害リスク最大
      </div>
      <SegmentedToggle<RiskLevel>
        aria-label="Maximum risk level"
        value={value}
        onChange={onChange}
        options={[
          {
            value: "low",
            label: "LOW",
            activeStyle: {
              background: GRADIENT.success,
              color: "#fff",
              border: "none",
            },
          },
          {
            value: "mid",
            label: "MID",
            activeStyle: {
              background: GRADIENT.warn,
              color: "#fff",
              border: "none",
            },
          },
          {
            value: "high",
            label: "HIGH",
            activeStyle: {
              background: GRADIENT.danger,
              color: "#fff",
              border: "none",
            },
          },
        ]}
      />
    </div>
  );
}

function PriceRangeSlider({
  value,
  onChange,
}: {
  value: [number, number];
  onChange: (v: [number, number]) => void;
}) {
  return (
    <div>
      <div className="mb-1.5 flex items-center justify-between text-[10px]">
        <span style={{ color: "var(--neutral-600)" }}>価格 ¥/㎡</span>
        <span
          className="font-extrabold"
          style={{ color: "var(--neutral-900)" }}
        >
          {formatYen(value[0])} - {formatYen(value[1])}
        </span>
      </div>
      {/* MVP: two separate sliders; full range slider component is later polish */}
      <div className="flex gap-2">
        <input
          type="range"
          min={0}
          max={5_000_000}
          step={100_000}
          value={value[0]}
          onChange={(e) =>
            onChange([Math.min(Number(e.target.value), value[1]), value[1]])
          }
          aria-label="Price minimum"
          className="flex-1"
        />
        <input
          type="range"
          min={0}
          max={10_000_000}
          step={100_000}
          value={value[1]}
          onChange={(e) =>
            onChange([value[0], Math.max(Number(e.target.value), value[0])])
          }
          aria-label="Price maximum"
          className="flex-1"
        />
      </div>
    </div>
  );
}

function formatYen(n: number): string {
  if (n >= 10_000) return `${Math.round(n / 10_000)}万`;
  return `¥${n}`;
}
