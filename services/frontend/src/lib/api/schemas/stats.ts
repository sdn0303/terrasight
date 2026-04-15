import { z } from "zod";

export const StatsResponse = z.object({
  land_price: z.object({
    avg_per_sqm: z.number(),
    median_per_sqm: z.number(),
    min_per_sqm: z.number(),
    max_per_sqm: z.number(),
    count: z.number(),
  }),
  risk: z.object({
    flood_area_ratio: z.number(),
    steep_slope_area_ratio: z.number(),
    composite_risk: z.number(),
  }),
  facilities: z.object({
    schools: z.number(),
    medical: z.number(),
  }),
  zoning_distribution: z.record(z.string(), z.number()),
});
export type StatsResponse = z.infer<typeof StatsResponse>;
