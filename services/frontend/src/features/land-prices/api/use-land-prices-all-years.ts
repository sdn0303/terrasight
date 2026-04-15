import { useQuery } from "@tanstack/react-query";
import { type BBox, typedGet } from "@/lib/api";
import { isBBoxValid } from "@/lib/api/bbox-guard";
import { LandPriceTimeSeriesResponse } from "@/lib/api/schemas/land-prices";
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
      const clampedZoom = Math.min(Math.floor(zoom), 22);
      return typedGet(
        LandPriceTimeSeriesResponse,
        "api/v1/land-prices/all-years",
        {
          bbox: `${bbox.west},${bbox.south},${bbox.east},${bbox.north}`,
          from: String(fromYear),
          to: String(toYear),
          zoom: String(clampedZoom),
        },
        signal,
      );
    },
    enabled: !!bbox && isBBoxValid(bbox) && zoom >= 10,
    staleTime: 300_000,
    retry: 1,
  });
}
