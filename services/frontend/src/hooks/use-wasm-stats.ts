"use client";

import { useQuery } from "@tanstack/react-query";
import { spatialEngine } from "@/lib/wasm/spatial-engine";
import type { BBox } from "@/lib/wasm/spatial-engine";
import { WasmStatsSchema } from "@/lib/api/schemas/wasm-stats";
import type { WasmStats } from "@/lib/api/schemas/wasm-stats";
import { useSpatialEngineState } from "@/hooks/use-spatial-engine";

export function useWasmStats(bbox: BBox | null) {
  const { ready } = useSpatialEngineState();

  return useQuery<WasmStats>({
    queryKey: ["wasm-stats", bbox?.south, bbox?.west, bbox?.north, bbox?.east],
    queryFn: async () => {
      const raw = await spatialEngine.computeStats(bbox!);
      return WasmStatsSchema.parse(raw);
    },
    enabled: bbox !== null && ready,
    staleTime: 5_000,
  });
}
