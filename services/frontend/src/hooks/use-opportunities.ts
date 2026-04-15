"use client";

import { useQuery } from "@tanstack/react-query";
import { type BBox, typedGet } from "@/lib/api";
import type { OpportunityRiskLevel } from "@/lib/api/schemas/opportunities";
import { OpportunitiesResponse } from "@/lib/api/schemas/opportunities";
import { queryKeys } from "@/lib/query-keys";
import { useFilterStore } from "@/stores/filter-store";
import { useMapStore } from "@/stores/map-store";

export interface FetchOpportunitiesParams {
  bbox: BBox;
  limit: number;
  offset: number;
  tlsMin?: number;
  riskMax?: OpportunityRiskLevel;
  zones?: string[];
  stationMax?: number;
  priceMin?: number;
  priceMax?: number;
  preset?: string;
}

/**
 * Fetches opportunities for the current viewport bbox, gated by `enabled`
 * (typically `bottomSheet === "opportunities"`).
 *
 * Filter-store fields are selected discretely so the query key remains
 * referentially stable across re-renders; only actual value changes
 * trigger a refetch.
 */
export function useOpportunities(enabled: boolean) {
  const bbox = useMapStore((s) => s.getBBox());

  const tlsMin = useFilterStore((s) => s.criteria.tlsMin);
  const riskMax = useFilterStore((s) => s.criteria.riskMax);
  const priceMin = useFilterStore((s) => s.criteria.priceRange[0]);
  const priceMax = useFilterStore((s) => s.criteria.priceRange[1]);
  const zones = useFilterStore((s) => s.zoning.zones);
  const stationMaxDistanceM = useFilterStore(
    (s) => s.zoning.stationMaxDistanceM,
  );
  const preset = useFilterStore((s) => s.preset);

  const zonesCsv = zones.join(",");

  // Normalise "inactive filter" sentinels into `undefined` so both the
  // query key and the HTTP request omit them.
  const tlsMinFilter = tlsMin > 0 ? tlsMin : undefined;
  const riskMaxFilter = riskMax !== "high" ? riskMax : undefined;
  const zonesFilter = zones.length > 0 ? zones : undefined;
  const stationMaxFilter =
    stationMaxDistanceM < 2000 ? stationMaxDistanceM : undefined;
  const priceMinFilter = priceMin > 0 ? priceMin : undefined;
  const priceMaxFilter = priceMax < 10_000_000 ? priceMax : undefined;
  const presetFilter = preset !== "balance" ? preset : undefined;

  return useQuery({
    queryKey: queryKeys.opportunities.list(bbox, {
      tlsMin: tlsMinFilter,
      riskMax: riskMaxFilter,
      zones: zonesCsv,
      stationMax: stationMaxFilter,
      priceMin: priceMinFilter,
      priceMax: priceMaxFilter,
      preset: presetFilter,
    }),
    queryFn: ({ signal }) => {
      const params: Record<string, string> = {
        bbox: `${bbox.west},${bbox.south},${bbox.east},${bbox.north}`,
        limit: "50",
        offset: "0",
      };
      if (tlsMinFilter !== undefined) params.tls_min = String(tlsMinFilter);
      if (riskMaxFilter !== undefined) params.risk_max = riskMaxFilter;
      if (zonesFilter !== undefined) params.zones = zonesFilter.join(",");
      if (stationMaxFilter !== undefined)
        params.station_max = String(stationMaxFilter);
      if (priceMinFilter !== undefined)
        params.price_min = String(priceMinFilter);
      if (priceMaxFilter !== undefined)
        params.price_max = String(priceMaxFilter);
      if (presetFilter !== undefined) params.preset = presetFilter;
      return typedGet(
        OpportunitiesResponse,
        "api/v1/opportunities",
        params,
        signal,
        { timeout: 60_000 },
      );
    },
    enabled,
    staleTime: 60_000,
    retry: 1,
  });
}
