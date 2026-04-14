import { z } from "zod";

export const AppraisalDetailSchema = z.object({
  city_code: z.string(),
  city_name: z.string(),
  address: z.string(),
  land_use_code: z.string(),
  price_per_sqm: z.number(),
  appraisal_price: z.number(),
  lot_area_sqm: z.number().nullable(),
  zone_code: z.string().nullable(),
  building_coverage: z.number().nullable(),
  floor_area_ratio: z.number().nullable(),
  comparable_price: z.number().nullable(),
  yield_price: z.number().nullable(),
  cost_price: z.number().nullable(),
  fudosan_id: z.string().nullable(),
});

export type AppraisalDetail = z.infer<typeof AppraisalDetailSchema>;
