"use client";

import { InfraProximity } from "@/components/analyze/infra-proximity";
import { useScore } from "@/features/score/api/use-score";

interface InfraTabProps {
  lat: number;
  lng: number;
}

export function InfraTab({ lat, lng }: InfraTabProps) {
  const { data: score, isLoading } = useScore(lat, lng, "residential");

  if (isLoading || !score) {
    return (
      <div role="status" aria-label="Loading infra">
        <div
          className="h-24 animate-pulse rounded-lg"
          style={{ background: "var(--neutral-100)" }}
        />
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <InfraProximity livabilityAxis={score.axes.livability} />
    </div>
  );
}
