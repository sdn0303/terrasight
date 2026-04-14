import { z } from "zod";

export const SignalSchema = z.enum(["hot", "warm", "neutral", "cold"]);
export type Signal = z.infer<typeof SignalSchema>;

export const OpportunityRiskLevelSchema = z.enum(["low", "mid", "high"]);
export type OpportunityRiskLevel = z.infer<typeof OpportunityRiskLevelSchema>;

export const OpportunityStationSchema = z.object({
  name: z.string(),
  distance_m: z.number().int().nonnegative(),
});

export const OpportunitySchema = z.object({
  id: z.number().int(),
  lat: z.number(),
  lng: z.number(),
  address: z.string(),
  zone: z.string(),
  building_coverage_ratio: z.number().int(),
  floor_area_ratio: z.number().int(),
  tls: z.number().int().min(0).max(100),
  risk_level: OpportunityRiskLevelSchema,
  trend_pct: z.number(),
  station: OpportunityStationSchema.nullable(),
  price_per_sqm: z.number().int().nonnegative(),
  signal: SignalSchema,
});
export type Opportunity = z.infer<typeof OpportunitySchema>;

export const OpportunitiesResponse = z.object({
  items: z.array(OpportunitySchema),
  total: z.number().int().nonnegative(),
  truncated: z.boolean(),
});
export type OpportunitiesResponse = z.infer<typeof OpportunitiesResponse>;
