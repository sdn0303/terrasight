import { useQuery } from "@tanstack/react-query";
import { z } from "zod";
import { typedGet } from "@/lib/api";
import { PopulationSchema } from "@/lib/api/schemas/population";
import { queryKeys } from "@/lib/query-keys";

const PopulationListSchema = z.array(PopulationSchema);

export function usePopulation(prefCode: string | null) {
  return useQuery({
    queryKey: prefCode
      ? queryKeys.population.byPref(prefCode)
      : queryKeys.population.all,
    queryFn: ({ signal }) => {
      if (prefCode === null) throw new Error("prefCode is required");
      return typedGet(
        PopulationListSchema,
        "api/v1/population",
        { pref_code: prefCode },
        signal,
      );
    },
    enabled: prefCode !== null,
    staleTime: 60_000,
    retry: 1,
  });
}
