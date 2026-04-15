import { z } from "zod";

export const AreaStatsResponse = z.object({
  code: z.string(),
  name: z.string(),
  level: z.enum(["prefecture", "municipality"]),
  land_price: z.object({
    avg_per_sqm: z.number().nullable(),
    median_per_sqm: z.number().nullable(),
    count: z.number(),
  }),
  risk: z.object({
    flood_area_ratio: z.number(),
    composite_risk: z.number(),
  }),
  facilities: z.object({
    schools: z.number(),
    medical: z.number(),
  }),
});
export type AreaStatsResponse = z.infer<typeof AreaStatsResponse>;
