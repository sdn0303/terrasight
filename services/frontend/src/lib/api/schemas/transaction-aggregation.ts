import { z } from "zod";

const TransactionAggregationProperties = z.object({
  admin_code: z.string(),
  city_name: z.string(),
  tx_count: z.number(),
  avg_price_sqm: z.number(),
  avg_total_price: z.number(),
});

const TransactionAggregationFeature = z.object({
  type: z.literal("Feature"),
  geometry: z.object({
    type: z.enum(["Polygon", "MultiPolygon"]),
    coordinates: z.unknown(),
  }),
  properties: TransactionAggregationProperties,
});

export const TransactionAggregationResponse = z.object({
  type: z.literal("FeatureCollection"),
  features: z.array(TransactionAggregationFeature),
});

export type TransactionAggregation = z.infer<
  typeof TransactionAggregationResponse
>;
