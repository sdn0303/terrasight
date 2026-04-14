"use client";

import { useQuery } from "@tanstack/react-query";
import { typedGet } from "@/lib/api";
import { TlsResponse } from "@/lib/api/schemas/score";
import { queryKeys } from "@/lib/query-keys";

/**
 * Fetches the TLS score for a specific lat/lng coordinate.
 * Wraps the shared score query key so the result can be read
 * from the cache by other consumers.
 */
export function useOpportunityScore(
  lat: number | null,
  lng: number | null,
  preset = "balance",
) {
  return useQuery({
    queryKey: queryKeys.score.coord(lat ?? 0, lng ?? 0, preset),
    queryFn: ({ signal }) => {
      if (lat === null) throw new Error("lat is required");
      if (lng === null) throw new Error("lng is required");
      return typedGet(
        TlsResponse,
        "api/v1/score",
        { lat: String(lat), lng: String(lng), preset },
        signal,
      );
    },
    enabled: lat !== null && lng !== null,
    staleTime: 60_000,
    retry: 1,
  });
}
