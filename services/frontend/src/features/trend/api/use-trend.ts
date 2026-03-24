import { useQuery } from "@tanstack/react-query";
import { fetchTrend } from "@/lib/api";
import { queryKeys } from "@/lib/query-keys";

export function useTrend(
  lat: number | null,
  lng: number | null,
  years?: number,
) {
  return useQuery({
    queryKey: queryKeys.trend.coord(lat ?? 0, lng ?? 0, years),
    queryFn: ({ signal }) => {
      if (lat === null) throw new Error("lat is required");
      if (lng === null) throw new Error("lng is required");
      return fetchTrend(lat, lng, years, signal);
    },
    enabled: lat !== null && lng !== null,
    staleTime: 60_000,
    retry: 1,
  });
}
