import { useQuery } from "@tanstack/react-query";
import { type BBox, fetchLandPrices } from "@/lib/api";
import { queryKeys } from "@/lib/query-keys";

export function useLandPrices(bbox: BBox | null, year: number, zoom: number) {
  return useQuery({
    queryKey: queryKeys.landPrices.byYear(
      bbox ?? { south: 0, west: 0, north: 0, east: 0 },
      year,
    ),
    queryFn: ({ signal }) => {
      if (bbox === null) throw new Error("bbox is required");
      return fetchLandPrices(bbox, year, zoom, signal);
    },
    enabled: bbox !== null && zoom >= 10,
    staleTime: 60_000,
    retry: 1,
  });
}
