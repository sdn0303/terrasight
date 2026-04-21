import { useQuery } from "@tanstack/react-query";
import { z } from "zod";
import { typedGet } from "@/lib/api";
import { VacancySchema } from "@/lib/api/schemas/vacancy";
import { queryKeys } from "@/lib/query-keys";

const VacancyListSchema = z.array(VacancySchema);

export function useVacancy(prefCode: string | null) {
  return useQuery({
    queryKey: prefCode
      ? queryKeys.vacancy.byPref(prefCode)
      : queryKeys.vacancy.all,
    queryFn: ({ signal }) => {
      if (prefCode === null) throw new Error("prefCode is required");
      return typedGet(
        VacancyListSchema,
        "api/v1/vacancy",
        { pref_code: prefCode },
        signal,
      );
    },
    enabled: prefCode !== null,
    staleTime: 60_000,
    retry: 1,
  });
}
