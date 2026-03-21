import { useQuery } from "@tanstack/react-query";
import { fetchStats, type BBox } from "@/lib/api";
import { queryKeys } from "@/lib/query-keys";

export function useStats(bbox: BBox | null) {
  return useQuery({
    queryKey: queryKeys.stats.bbox(
      bbox ?? { south: 0, west: 0, north: 0, east: 0 },
    ),
    queryFn: () => fetchStats(bbox!),
    enabled: bbox !== null,
  });
}
