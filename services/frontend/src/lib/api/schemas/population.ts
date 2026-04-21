import { z } from "zod";

export const PopulationSchema = z.object({
  city_code: z.string(),
  city_name: z.string(),
  population: z.number(),
  male: z.number().nullable(),
  female: z.number().nullable(),
  households: z.number().nullable(),
  census_year: z.number(),
});

export type Population = z.infer<typeof PopulationSchema>;
