"use client";

import { useQuery } from "@tanstack/react-query";
import { spatialEngine } from "@/lib/wasm/spatial-engine";
import type { BBox } from "@/lib/wasm/spatial-engine";
import { useSpatialEngineState } from "@/hooks/use-spatial-engine";

export function useWasmTls(bbox: BBox | null, preset = "balance") {
  const { ready } = useSpatialEngineState();

  return useQuery({
    queryKey: [
      "wasm-tls",
      bbox?.south,
      bbox?.west,
      bbox?.north,
      bbox?.east,
      preset,
    ],
    queryFn: async () => {
      const raw = await spatialEngine.computeTls(bbox!, preset);
      return raw; // TODO: Zod validation once TLS schema is finalised
    },
    enabled: bbox !== null && ready,
    staleTime: 5_000,
  });
}
