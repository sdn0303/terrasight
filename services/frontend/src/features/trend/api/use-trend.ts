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
    queryFn: () => fetchTrend(lat!, lng!, years),
    enabled: lat !== null && lng !== null,
  });
}
