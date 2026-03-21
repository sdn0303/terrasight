import { useQuery } from "@tanstack/react-query";
import { fetchScore } from "@/lib/api";
import { queryKeys } from "@/lib/query-keys";

export function useScore(lat: number | null, lng: number | null) {
  return useQuery({
    queryKey: queryKeys.score.coord(lat ?? 0, lng ?? 0),
    queryFn: () => {
      if (lat === null) throw new Error("lat is required");
      if (lng === null) throw new Error("lng is required");
      return fetchScore(lat, lng);
    },
    enabled: lat !== null && lng !== null,
  });
}
