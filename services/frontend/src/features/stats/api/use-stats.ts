"use client";

import { useQuery } from "@tanstack/react-query";
import { fetchStats, type BBox } from "@/lib/api";
import { queryKeys } from "@/lib/query-keys";
import { StatsResponse } from "@/lib/schemas";
import { spatialEngine } from "@/lib/wasm/spatial-engine";
import { useSpatialEngineReady } from "@/hooks/use-spatial-engine";

export function useStats(bbox: BBox | null, zoom: number) {
  const wasmReady = useSpatialEngineReady();

  // WASM path: instant local computation using in-memory R-tree index.
  // The result is validated with the same StatsResponse Zod schema so
  // both paths return an identical, narrowed type.
  const wasmResult = useQuery({
    queryKey: ["stats-wasm", bbox?.south, bbox?.west, bbox?.north, bbox?.east],
    queryFn: async () => {
      if (!bbox) throw new Error("bbox required");
      const raw = await spatialEngine.computeStats(bbox);
      return StatsResponse.parse(raw);
    },
    enabled: bbox !== null && zoom >= 10 && wasmReady,
    staleTime: 5_000,
  });

  // API fallback: used when WASM is not initialised or unavailable.
  const apiResult = useQuery({
    queryKey: queryKeys.stats.bbox(bbox ?? { south: 0, west: 0, north: 0, east: 0 }),
    queryFn: ({ signal }) => {
      if (!bbox) throw new Error("bbox required");
      return fetchStats(bbox, signal);
    },
    enabled: bbox !== null && zoom >= 10 && !wasmReady,
    staleTime: 60_000,
    retry: 1,
  });

  return wasmReady ? wasmResult : apiResult;
}
