import { useQuery } from "@tanstack/react-query";
import { typedGet } from "@/lib/api";
import { isBBoxValid } from "@/lib/api/bbox-guard";
import { LandPriceAggregationResponse } from "@/lib/api/schemas/land-price-aggregation";
import { queryKeys } from "@/lib/query-keys";
import { useMapStore } from "@/stores/map-store";

/**
 * Fetches city-level land price aggregation polygons for the current viewport.
 * Only active when zoom < 14 (polygon view) — at higher zoom the point layer
 * takes over. staleTime is 2 minutes as aggregated data changes infrequently.
 */
export function useLandPriceAggregation() {
  // Subscribe to viewState for reactivity; derive bbox via getState() to avoid
  // creating a new object reference on every render (request flood prevention).
  const viewState = useMapStore((s) => s.viewState);
  const bbox = useMapStore.getState().getBBox();
  // viewState is referenced to trigger re-subscription when the map moves.
  void viewState;

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
    enabled: isBBoxValid(bbox) && viewState.zoom < 14,
    staleTime: 120_000,
  });
}
