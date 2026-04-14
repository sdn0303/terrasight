import { useQuery } from "@tanstack/react-query";
import { fetchHealth } from "@/lib/api";
import { queryKeys } from "@/lib/query-keys";

export function useHealth() {
  return useQuery({
    queryKey: queryKeys.health,
    queryFn: ({ signal }) => fetchHealth(signal),
    staleTime: 30_000,
  });
}
