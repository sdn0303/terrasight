"use client";

import type { Signal } from "@/lib/schemas";
import { GLOW_SHADOW, signalGradient } from "@/lib/theme-tokens";

interface StatusPillProps {
  status: Signal;
}

const LABELS: Record<Signal, string> = {
  hot: "Hot",
  warm: "Warm",
  neutral: "Neutral",
  cold: "Cold",
};

/**
 * Pill showing an opportunity signal tier (Hot/Warm/Neutral/Cold).
 * Hot/warm use the gradient background with white text; neutral/cold use
 * lighter backgrounds with neutral-700 text.
 */
export function StatusPill({ status }: StatusPillProps) {
  const isHot = status === "hot";
  const isWarm = status === "warm";
  const color = isHot || isWarm ? "#fff" : "var(--neutral-700)";
  const glow = isHot ? GLOW_SHADOW.hot : undefined;

  return (
    <span
      className="inline-block rounded-full text-[9px] font-extrabold uppercase"
      style={{
        background: signalGradient(status),
        color,
        padding: "4px 12px",
        letterSpacing: "0.5px",
        boxShadow: glow,
      }}
    >
      {isHot && "🔥 "}
      {LABELS[status]}
    </span>
  );
}
