

import { PageChip } from "@/components/layout/page-chip";

interface StatusChipProps {
  lat: number;
  lng: number;
  zoom: number;
  featureCount: number;
  cacheState?: "fresh" | "cached" | "stale" | undefined;
}

export function StatusChip({
  lat,
  lng,
  zoom,
  featureCount,
  cacheState,
}: StatusChipProps) {
  return (
    <PageChip anchor="bottom-right" aria-label="Map status">
      <div
        className="text-[9px]"
        style={{
          color: "var(--neutral-600)",
          fontFamily: "var(--font-mono)",
        }}
      >
        <div>
          <span style={{ color: "var(--neutral-900)", fontWeight: 700 }}>
            {lat.toFixed(4)}, {lng.toFixed(4)}
          </span>{" "}
          · z{zoom.toFixed(0)}
        </div>
        <div
          className="mt-0.5 text-[8px]"
          style={{ color: "var(--neutral-400)" }}
        >
          {featureCount.toLocaleString("en-US")} features
          {cacheState !== undefined && ` · ${cacheState}`}
        </div>
      </div>
    </PageChip>
  );
}
