import { useQuery } from "@tanstack/react-query";
import { useDebouncedValue } from "@/hooks/use-debounced-value";
import { typedGet } from "@/lib/api";
import { isBBoxValid } from "@/lib/api/bbox-guard";
import { TransactionAggregationResponse } from "@/lib/api/schemas/transaction-aggregation";
import { queryKeys } from "@/lib/query-keys";
import { useMapStore } from "@/stores/map-store";

export function useTransactionAggregation() {
  const viewState = useMapStore((s) => s.viewState);
  const debouncedViewState = useDebouncedValue(viewState, 300);
  const bbox = useMapStore.getState().getBBox();

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
    // Only fetch at low zoom (polygon view) and after the debounce settles.
    enabled: isBBoxValid(bbox) && debouncedViewState.zoom < 14,
    // Aggregated city-level data changes infrequently; 2-minute stale window
    staleTime: 120_000,
    retry: 1,
  });
}
