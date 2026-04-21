import { z } from "zod";

export const VacancySchema = z.object({
  city_code: z.string(),
  city_name: z.string(),
  vacancy_count: z.number(),
  total_houses: z.number().nullable(),
  vacancy_rate_pct: z.number().nullable(),
  survey_year: z.number(),
});

export type Vacancy = z.infer<typeof VacancySchema>;
