"use client";

import { RiskBreakdown } from "@/components/analyze/risk-breakdown";
import { useScore } from "@/features/score/api/use-score";

interface RiskTabProps {
  lat: number;
  lng: number;
}

export function RiskTab({ lat, lng }: RiskTabProps) {
  const { data: score, isLoading } = useScore(lat, lng, "disaster");

  if (isLoading || !score) {
    return (
      <div role="status" aria-label="Loading risk">
        <div
          className="h-24 animate-pulse rounded-lg"
          style={{ background: "var(--neutral-100)" }}
        />
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <RiskBreakdown disasterAxis={score.axes.disaster} />
    </div>
  );
}
