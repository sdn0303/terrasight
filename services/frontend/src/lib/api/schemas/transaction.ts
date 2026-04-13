import { z } from "zod";

export const TransactionSummarySchema = z.object({
  city_code: z.string(),
  transaction_year: z.number(),
  property_type: z.string(),
  tx_count: z.number(),
  avg_total_price: z.number(),
  median_total_price: z.number(),
  avg_price_sqm: z.number().nullable(),
  avg_area_sqm: z.number().nullable(),
  avg_walk_min: z.number().nullable(),
});

export type TransactionSummary = z.infer<typeof TransactionSummarySchema>;

export const TransactionDetailSchema = z.object({
  city_code: z.string(),
  city_name: z.string(),
  district_name: z.string().nullable(),
  property_type: z.string(),
  total_price: z.number(),
  price_per_sqm: z.number().nullable(),
  area_sqm: z.number().nullable(),
  floor_plan: z.string().nullable(),
  building_year: z.number().nullable(),
  building_structure: z.string().nullable(),
  nearest_station: z.string().nullable(),
  station_walk_min: z.number().nullable(),
  transaction_quarter: z.string(),
});

export type TransactionDetail = z.infer<typeof TransactionDetailSchema>;
