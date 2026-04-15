import { z } from "zod";
import { layerResponse } from "@/lib/schemas";

export const LandPriceProperties = z.object({
  id: z.number(),
  price_per_sqm: z.number(),
  address: z.string(),
  land_use: z.string().nullable(),
  year: z.number(),
});

export type LandPriceProperties = z.infer<typeof LandPriceProperties>;

export const LandPriceTimeSeriesResponse = layerResponse(LandPriceProperties);
export type LandPriceTimeSeriesResponse = z.infer<
  typeof LandPriceTimeSeriesResponse
>;
