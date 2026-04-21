import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import { useDebouncedValue } from "@/hooks/use-debounced-value";
import { typedGet } from "@/lib/api";
import { isBBoxValid } from "@/lib/api/bbox-guard";
import { TransactionAggregationResponse } from "@/lib/api/schemas/transaction-aggregation";
import { queryKeys } from "@/lib/query-keys";
import { useMapStore } from "@/stores/map-store";

export function useTransactionAggregation() {
  // Select primitive values — stable under Object.is, only re-render
  // when the actual number changes (not on unrelated store updates).
  const rawLat = useMapStore((s) => s.viewState.latitude);
  const rawLng = useMapStore((s) => s.viewState.longitude);
  const rawZoom = useMapStore((s) => s.viewState.zoom);
  const latitude = useDebouncedValue(rawLat, 300);
  const longitude = useDebouncedValue(rawLng, 300);
  const zoom = useDebouncedValue(rawZoom, 300);

  // Derive bbox from the debounced primitives so that both queryKey and
  // queryFn use the settled position — no fetches during active panning.
  const bbox = useMemo(() => {
    const latRange = 180 / 2 ** zoom;
    const lngRange = 360 / 2 ** zoom;
    return {
      south: latitude - latRange / 2,
      west: longitude - lngRange / 2,
      north: latitude + latRange / 2,
      east: longitude + lngRange / 2,
    };
  }, [latitude, longitude, zoom]);

  return useQuery({
    queryKey: queryKeys.transactions.aggregation(bbox),
    queryFn: ({ signal }) =>
      typedGet(
        TransactionAggregationResponse,
        "api/v1/transactions/aggregation",
        {
          south: String(bbox.south),
          west: String(bbox.west),
          north: String(bbox.north),
          east: String(bbox.east),
        },
        signal,
      ),
    enabled: isBBoxValid(bbox) && zoom < 14,
    // Aggregated city-level data changes infrequently; 2-minute stale window
    staleTime: 120_000,
    retry: 1,
  });
}
