import { useQuery } from "@tanstack/react-query";
import { typedGet } from "@/lib/api";
import { TlsResponse } from "@/lib/schemas";
import { queryKeys } from "@/lib/query-keys";

export function useScore(
  lat: number | null,
  lng: number | null,
  preset: string = "balance",
) {
  return useQuery({
    queryKey: queryKeys.score.coord(lat ?? 0, lng ?? 0, preset),
    queryFn: ({ signal }) => {
      if (lat === null) throw new Error("lat is required");
      if (lng === null) throw new Error("lng is required");
      return typedGet(
        TlsResponse,
        "api/v1/score",
        { lat: String(lat), lng: String(lng), preset },
        signal,
      );
    },
    enabled: lat !== null && lng !== null,
    staleTime: 60_000,
    retry: 1,
  });
}
