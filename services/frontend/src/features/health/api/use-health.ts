import { useQuery } from "@tanstack/react-query";
import { typedGet } from "@/lib/api";
import { HealthResponse } from "@/lib/api/schemas/health";
import { queryKeys } from "@/lib/query-keys";

export function useHealth() {
  return useQuery({
    queryKey: queryKeys.health,
    queryFn: ({ signal }) =>
      typedGet(HealthResponse, "api/v1/health", undefined, signal),
    staleTime: 30_000,
  });
}
