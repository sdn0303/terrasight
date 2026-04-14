"use client";

import { useScore } from "@/features/score/api/use-score";
import { AxisBreakdown } from "./axis-breakdown";
import { ScoreHeroCard } from "./score-hero-card";

interface IntelTabProps {
  lat: number;
  lng: number;
}

export function IntelTab({ lat, lng }: IntelTabProps) {
  // Balance preset drives the hero card; dedicated preset selection will
  // be wired in Phase 3 when the multi-preset endpoint is available.
  const { data: score, isLoading } = useScore(lat, lng, "balance");

  if (isLoading || !score) {
    return (
      <div role="status" aria-label="Loading intel">
        <div
          className="h-24 animate-pulse rounded-lg"
          style={{ background: "var(--neutral-100)" }}
        />
      </div>
    );
  }

  const presetStats = {
    balance: Math.round(score.tls.score),
    // Placeholder: real values come from a multi-preset endpoint in Phase 3.
    residential: Math.round(score.tls.score * 0.98),
    disaster: Math.round(score.axes.disaster.score),
  };

  return (
    <div className="space-y-4">
      <ScoreHeroCard
        tls={Math.round(score.tls.score)}
        topPercentile={null}
        deltaVsArea={0}
        presetStats={presetStats}
      />
      <AxisBreakdown
        axes={{
          disaster: Math.round(score.axes.disaster.score),
          terrain: Math.round(score.axes.terrain.score),
          livability: Math.round(score.axes.livability.score),
          future: Math.round(score.axes.future.score),
          price: Math.round(score.axes.price.score),
        }}
      />
    </div>
  );
}
