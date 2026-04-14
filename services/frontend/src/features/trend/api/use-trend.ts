import { useQuery } from "@tanstack/react-query";
import { typedGet } from "@/lib/api";
import { TrendResponse } from "@/lib/schemas";
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
      const params: Record<string, string> = {
        lat: String(lat),
        lng: String(lng),
      };
      if (years !== undefined) {
        params.years = String(years);
      }
      return typedGet(TrendResponse, "api/v1/trend", params, signal);
    },
    enabled: lat !== null && lng !== null,
    staleTime: 60_000,
    retry: 1,
  });
}
