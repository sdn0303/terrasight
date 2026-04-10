"use client";

import { GLOW_SHADOW, GRADIENT, scoreGradient } from "@/lib/theme-tokens";

interface ScoreHeroCardProps {
  tls: number;
  topPercentile: number | null;
  deltaVsArea: number;
  presetStats: {
    balance: number;
    residential: number;
    disaster: number;
  };
}

export function ScoreHeroCard({
  tls,
  topPercentile,
  deltaVsArea,
  presetStats,
}: ScoreHeroCardProps) {
  const deltaSign = deltaVsArea >= 0 ? "+" : "";
  const deltaArrow = deltaVsArea > 0 ? "▲" : deltaVsArea < 0 ? "▼" : "—";
  const deltaColor =
    deltaVsArea > 0
      ? "var(--score-good-start)"
      : deltaVsArea < 0
        ? "var(--score-bad-start)"
        : "var(--neutral-400)";

  return (
    <div
      className="p-3.5"
      style={{
        background: GRADIENT.heroCard,
        borderRadius: 14,
        border: "1px solid rgba(99, 102, 241, 0.1)",
      }}
    >
      <div className="flex items-center justify-between">
        <span
          className="text-[8px] font-extrabold uppercase"
          style={{
            color: "var(--brand-indigo)",
            letterSpacing: "0.7px",
          }}
        >
          TLS Score
        </span>
        {topPercentile !== null && (
          <span
            className="rounded-full px-2 py-0.5 text-[7px] font-extrabold uppercase text-white"
            style={{
              background: GRADIENT.success,
              boxShadow: GLOW_SHADOW.success,
            }}
          >
            Top {topPercentile}%
          </span>
        )}
      </div>
      <div className="mt-1 flex items-baseline gap-1.5">
        <span
          className="text-[34px] font-extrabold leading-none"
          style={{
            background: scoreGradient(tls),
            WebkitBackgroundClip: "text",
            backgroundClip: "text",
            color: "transparent",
            letterSpacing: "-1px",
          }}
        >
          {tls}
        </span>
        <span className="text-[10px] font-bold" style={{ color: deltaColor }}>
          {deltaArrow} {deltaSign}
          {deltaVsArea} vs area
        </span>
      </div>
      <div className="mt-2.5 flex gap-1.5">
        <PresetMiniStat label="Balance" value={presetStats.balance} />
        <PresetMiniStat label="Resid." value={presetStats.residential} />
        <PresetMiniStat label="Disaster" value={presetStats.disaster} />
      </div>
    </div>
  );
}

function PresetMiniStat({ label, value }: { label: string; value: number }) {
  return (
    <div
      className="flex-1 rounded-[10px] py-1.5 text-center"
      style={{ background: "rgba(255,255,255,0.7)" }}
    >
      <div
        className="text-[8px] uppercase"
        style={{ color: "var(--neutral-400)" }}
      >
        {label}
      </div>
      <div
        className="text-[11px] font-extrabold"
        style={{ color: "var(--neutral-900)" }}
      >
        {value}
      </div>
    </div>
  );
}
