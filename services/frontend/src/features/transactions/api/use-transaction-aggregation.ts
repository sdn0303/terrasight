import { useQuery } from "@tanstack/react-query";
import { typedGet } from "@/lib/api";
import { isBBoxValid } from "@/lib/api/bbox-guard";
import { TransactionAggregationResponse } from "@/lib/api/schemas/transaction-aggregation";
import { queryKeys } from "@/lib/query-keys";
import { useMapStore } from "@/stores/map-store";

/**
 * Fetches city-level transaction aggregation polygons for the current viewport.
 * Only active when zoom < 14 (polygon view). staleTime is 2 minutes as
 * aggregated data changes infrequently.
 */
export function useTransactionAggregation() {
  // Subscribe to viewState for reactivity; derive bbox via getState() to avoid
  // creating a new object reference on every render (request flood prevention).
  const viewState = useMapStore((s) => s.viewState);
  const bbox = useMapStore.getState().getBBox();
  // viewState is referenced to trigger re-subscription when the map moves.
  void viewState;

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
    enabled: isBBoxValid(bbox) && viewState.zoom < 14,
    staleTime: 120_000,
  });
}
