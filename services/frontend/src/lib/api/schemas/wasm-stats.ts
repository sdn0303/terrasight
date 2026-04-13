import { z } from "zod";

const ZoningEntrySchema = z.object({
  zone: z.string(),
  ratio: z.number(),
});

export const WasmStatsSchema = z.object({
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
    stations_nearby: z.number(),
  }),
  zoning_distribution: z.array(ZoningEntrySchema),
});

export type WasmStats = z.infer<typeof WasmStatsSchema>;
