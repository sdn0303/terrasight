import { z } from "zod";

export const MunicipalitySchema = z.object({
  city_code: z.string(),
  city_name: z.string(),
  pref_code: z.string(),
});

export type Municipality = z.infer<typeof MunicipalitySchema>;
