import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import { useDebouncedValue } from "@/hooks/use-debounced-value";
import { typedGet } from "@/lib/api";
import { isBBoxValid } from "@/lib/api/bbox-guard";
import { LandPriceAggregationResponse } from "@/lib/api/schemas/land-price-aggregation";
import { queryKeys } from "@/lib/query-keys";
import { useMapStore } from "@/stores/map-store";

export function useLandPriceAggregation() {
  const viewState = useMapStore((s) => s.viewState);
  const debouncedVS = useDebouncedValue(viewState, 300);

  // Derive bbox from the debounced viewState so that both queryKey and
  // queryFn use the settled position — no fetches during active panning.
  const bbox = useMemo(() => {
    const latRange = 180 / 2 ** debouncedVS.zoom;
    const lngRange = 360 / 2 ** debouncedVS.zoom;
    return {
      south: debouncedVS.latitude - latRange / 2,
      west: debouncedVS.longitude - lngRange / 2,
      north: debouncedVS.latitude + latRange / 2,
      east: debouncedVS.longitude + lngRange / 2,
    };
  }, [debouncedVS.latitude, debouncedVS.longitude, debouncedVS.zoom]);

  return useQuery({
    queryKey: queryKeys.landPrices.aggregation(bbox),
    queryFn: ({ signal }) =>
      typedGet(
        LandPriceAggregationResponse,
        "api/v1/land-prices/aggregation",
        {
          south: String(bbox.south),
          west: String(bbox.west),
          north: String(bbox.north),
          east: String(bbox.east),
        },
        signal,
      ),
    enabled: isBBoxValid(bbox) && debouncedVS.zoom < 14,
    // Aggregated city-level data changes infrequently; 2-minute stale window
    staleTime: 120_000,
    retry: 1,
  });
}
