import { z } from "zod";

export const TrendResponse = z.object({
  location: z.object({
    address: z.string(),
    distance_m: z.number(),
  }),
  data: z.array(
    z.object({
      year: z.number(),
      price_per_sqm: z.number(),
    }),
  ),
  cagr: z.number(),
  direction: z.enum(["up", "down"]),
});
export type TrendResponse = z.infer<typeof TrendResponse>;
