import { useQuery } from "@tanstack/react-query";
import { type BBox, typedGet } from "@/lib/api";
import { isBBoxValid } from "@/lib/api/bbox-guard";
import { LandPriceTimeSeriesResponse } from "@/lib/api/schemas/land-prices";
import { queryKeys } from "@/lib/query-keys";

export function useLandPrices(bbox: BBox | null, year: number, zoom: number) {
  return useQuery({
    queryKey: queryKeys.landPrices.byYear(
      bbox ?? { south: 0, west: 0, north: 0, east: 0 },
      year,
    ),
    queryFn: ({ signal }) => {
      if (bbox === null) throw new Error("bbox is required");
      const clampedZoom = Math.min(Math.floor(zoom), 22);
      return typedGet(
        LandPriceTimeSeriesResponse,
        "api/v1/land-prices",
        {
          south: String(bbox.south),
          west: String(bbox.west),
          north: String(bbox.north),
          east: String(bbox.east),
          year: String(year),
          zoom: String(clampedZoom),
        },
        signal,
      );
    },
    enabled: !!bbox && isBBoxValid(bbox) && zoom >= 10,
    staleTime: 60_000,
    retry: 1,
  });
}
