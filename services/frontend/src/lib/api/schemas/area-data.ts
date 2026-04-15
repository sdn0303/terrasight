import { z } from "zod";
import { LandPriceProperties } from "@/lib/api/schemas/land-prices";
import { layerResponse } from "@/lib/schemas";

export const ZoningProperties = z.object({
  id: z.number(),
  zone_type: z.string(),
  zone_code: z.string(),
  floor_area_ratio: z.number(),
  building_coverage: z.number(),
});

export const FloodProperties = z.object({
  id: z.number(),
  depth_rank: z.number(), // 0=outside zone, 1=<0.5m, 2=0.5-3m, 3=3-5m, 4=5-10m, 5=≥10m
  river_name: z.string().nullable(),
});

export const SteepSlopeProperties = z.object({
  id: z.number(),
  area_name: z.string(),
});

export const SchoolProperties = z.object({
  id: z.number(),
  name: z.string(),
  school_type: z.string(),
});

export const MedicalProperties = z.object({
  id: z.number(),
  name: z.string(),
  facility_type: z.string(),
  bed_count: z.number(),
});

export type ZoningProperties = z.infer<typeof ZoningProperties>;
export type FloodProperties = z.infer<typeof FloodProperties>;
export type SteepSlopeProperties = z.infer<typeof SteepSlopeProperties>;
export type SchoolProperties = z.infer<typeof SchoolProperties>;
export type MedicalProperties = z.infer<typeof MedicalProperties>;

export const AreaDataResponse = z.object({
  landprice: layerResponse(LandPriceProperties).optional(),
  zoning: layerResponse(ZoningProperties).optional(),
  flood: layerResponse(FloodProperties).optional(),
  steep_slope: layerResponse(SteepSlopeProperties).optional(),
  schools: layerResponse(SchoolProperties).optional(),
  medical: layerResponse(MedicalProperties).optional(),
});
export type AreaDataResponse = z.infer<typeof AreaDataResponse>;
