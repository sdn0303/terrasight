"use client";

import { useQuery } from "@tanstack/react-query";
import { fetchOpportunities } from "@/lib/api";
import { queryKeys } from "@/lib/query-keys";
import { useFilterStore } from "@/stores/filter-store";
import { useMapStore } from "@/stores/map-store";

/**
 * Fetches opportunities for the current viewport bbox, gated by `enabled`
 * (typically `bottomSheet === "opportunities"`).
 *
 * Filter-store fields are selected discretely so the query key remains
 * referentially stable across re-renders; only actual value changes
 * trigger a refetch.
 */
export function useOpportunities(enabled: boolean) {
  // Subscribe to viewState so the component re-renders on pan/zoom, then
  // call getBBox() unconditionally. The query key serialization handles
  // structural comparison: bbox identity changes are harmless.
  const viewState = useMapStore((s) => s.viewState);
  const bbox = useMapStore.getState().getBBox();
  // `viewState` is intentionally referenced to force re-subscription even
  // though the derived bbox is read via `getState()`.
  void viewState;

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
    queryFn: ({ signal }) =>
      fetchOpportunities(
        {
          bbox,
          limit: 50,
          offset: 0,
          tlsMin: tlsMinFilter,
          riskMax: riskMaxFilter,
          zones: zonesFilter,
          stationMax: stationMaxFilter,
          priceMin: priceMinFilter,
          priceMax: priceMaxFilter,
          preset: presetFilter,
        },
        signal,
      ),
    enabled,
    staleTime: 60_000,
    retry: 1,
  });
}
