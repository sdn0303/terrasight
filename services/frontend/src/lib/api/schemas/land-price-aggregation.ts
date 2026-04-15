import { z } from "zod";

const LandPriceAggregationProperties = z.object({
  admin_code: z.string(),
  pref_name: z.string(),
  city_name: z.string(),
  avg_price: z.number(),
  median_price: z.number(),
  min_price: z.number(),
  max_price: z.number(),
  count: z.number(),
  prev_year_avg: z.number(),
  change_pct: z.number(),
});

const LandPriceAggregationFeature = z.object({
  type: z.literal("Feature"),
  geometry: z.object({
    type: z.enum(["Polygon", "MultiPolygon"]),
    coordinates: z.unknown(),
  }),
  properties: LandPriceAggregationProperties,
});

export const LandPriceAggregationResponse = z.object({
  type: z.literal("FeatureCollection"),
  features: z.array(LandPriceAggregationFeature),
});

export type LandPriceAggregation = z.infer<typeof LandPriceAggregationResponse>;
