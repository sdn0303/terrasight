import { useQuery } from "@tanstack/react-query";
import { fetchAreaStats } from "@/lib/api";
import { queryKeys } from "@/lib/query-keys";

export function useAreaStats(code: string | null) {
  return useQuery({
    queryKey: queryKeys.areaStats.byCode(code ?? ""),
    queryFn: ({ signal }) => {
      if (code === null) throw new Error("code is required");
      return fetchAreaStats(code, signal);
    },
    enabled: code !== null,
    staleTime: 300_000, // 5 minutes — area stats don't change often
    retry: 1,
  });
}
