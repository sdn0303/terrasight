import { z } from "zod";

export const PopulationSchema = z.object({
  city_code: z.string(),
  city_name: z.string(),
  population: z.number(),
  male: z.number(),
  female: z.number(),
  households: z.number(),
  census_year: z.number(),
});

export type Population = z.infer<typeof PopulationSchema>;
