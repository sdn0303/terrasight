"use client";

import { useQuery } from "@tanstack/react-query";
import { fetchStats, type BBox } from "@/lib/api";
import { queryKeys } from "@/lib/query-keys";

// X-02 fix: WASM stats path disabled — partial layer load produces silent
// miscalculation (ready=true with missing layers → 0 values).
// /api/stats is the canonical source until WASM required-layer validation
// is implemented. See: docs/reviews/2026-03-27-frontend-wasm-backend-db-audit.md
export function useStats(bbox: BBox | null, zoom: number) {
  return useQuery({
    queryKey: queryKeys.stats.bbox(bbox ?? { south: 0, west: 0, north: 0, east: 0 }),
    queryFn: ({ signal }) => {
      if (!bbox) throw new Error("bbox required");
      return fetchStats(bbox, signal);
    },
    enabled: bbox !== null && zoom >= 10,
    staleTime: 60_000,
    retry: 1,
  });
}
