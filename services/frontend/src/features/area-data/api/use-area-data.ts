import { useQuery } from "@tanstack/react-query";
import { fetchAreaData, type BBox } from "@/lib/api";
import { queryKeys } from "@/lib/query-keys";

export function useAreaData(bbox: BBox | null, layers: string[]) {
  return useQuery({
    queryKey: queryKeys.areaData.bbox(
      bbox ?? { south: 0, west: 0, north: 0, east: 0 },
      layers,
    ),
    queryFn: () => {
      if (bbox === null) throw new Error("bbox is required");
      return fetchAreaData(bbox, layers);
    },
    enabled: bbox !== null && layers.length > 0,
    staleTime: 60_000,
  });
}
