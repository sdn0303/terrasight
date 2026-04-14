import { useQuery } from "@tanstack/react-query";
import { type BBox, typedGet } from "@/lib/api";
import { AreaDataResponse } from "@/lib/api/schemas/area-data";
import { queryKeys } from "@/lib/query-keys";

export function useAreaData(bbox: BBox | null, layers: string[], zoom: number) {
  return useQuery({
    queryKey: queryKeys.areaData.bbox(
      bbox ?? { south: 0, west: 0, north: 0, east: 0 },
      layers,
    ),
    queryFn: ({ signal }) => {
      if (bbox === null) throw new Error("bbox is required");
      const clampedZoom = Math.min(Math.floor(zoom), 22);
      return typedGet(
        AreaDataResponse,
        "api/v1/area-data",
        {
          south: String(bbox.south),
          west: String(bbox.west),
          north: String(bbox.north),
          east: String(bbox.east),
          layers: layers.join(","),
          zoom: String(clampedZoom),
        },
        signal,
      );
    },
    enabled: bbox !== null && layers.length > 0 && zoom >= 10,
    staleTime: 60_000,
  });
}
