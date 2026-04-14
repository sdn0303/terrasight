import { useQuery } from "@tanstack/react-query";
import { type BBox, fetchLandPricesAllYears } from "@/lib/api";
import { queryKeys } from "@/lib/query-keys";

/**
 * Fetch land prices across a year range in a single request. Used by the time
 * machine slider so that scrubbing between years filters the already-loaded
 * source client-side (via MapLibre `setFilter`) instead of refetching.
 *
 * Multi-year data changes rarely, so `staleTime` is set to 5 minutes.
 */
export function useLandPricesAllYears(
  bbox: BBox | null,
  fromYear: number,
  toYear: number,
  zoom: number,
) {
  return useQuery({
    queryKey: queryKeys.landPrices.allYears(
      bbox ?? { south: 0, west: 0, north: 0, east: 0 },
      fromYear,
      toYear,
    ),
    queryFn: ({ signal }) => {
      if (bbox === null) throw new Error("bbox is required");
      return fetchLandPricesAllYears(bbox, fromYear, toYear, zoom, signal);
    },
    enabled: bbox !== null && zoom >= 10,
    staleTime: 300_000,
    retry: 1,
  });
}
