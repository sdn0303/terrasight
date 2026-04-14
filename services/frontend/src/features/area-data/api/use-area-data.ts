import { useQuery } from "@tanstack/react-query";
import { type BBox, fetchAreaData } from "@/lib/api";
import { queryKeys } from "@/lib/query-keys";

export function useAreaData(bbox: BBox | null, layers: string[], zoom: number) {
  return useQuery({
    queryKey: queryKeys.areaData.bbox(
      bbox ?? { south: 0, west: 0, north: 0, east: 0 },
      layers,
    ),
    queryFn: ({ signal }) => {
      if (bbox === null) throw new Error("bbox is required");
      return fetchAreaData(bbox, layers, zoom, signal);
    },
    enabled: bbox !== null && layers.length > 0 && zoom >= 10,
    staleTime: 60_000,
  });
}
