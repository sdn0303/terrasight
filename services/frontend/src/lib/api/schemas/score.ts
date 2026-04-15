import { z } from "zod";

const SubScoreDto = z.object({
  id: z.string(),
  score: z.number(),
  available: z.boolean(),
  detail: z.record(z.string(), z.union([z.number(), z.boolean(), z.string()])),
});

const AxisDto = z.object({
  score: z.number(),
  weight: z.number(),
  confidence: z.number(),
  sub: z.array(SubScoreDto),
});

export const TlsResponse = z.object({
  location: z.object({
    lat: z.number(),
    lng: z.number(),
  }),
  tls: z.object({
    score: z.number(),
    grade: z.enum(["S", "A", "B", "C", "D", "E"]),
    label: z.string(),
  }),
  axes: z.object({
    disaster: AxisDto,
    terrain: AxisDto,
    livability: AxisDto,
    future: AxisDto,
    price: AxisDto,
  }),
  cross_analysis: z.object({
    value_discovery: z.number(),
    demand_signal: z.number(),
    ground_safety: z.number(),
  }),
  metadata: z.object({
    calculated_at: z.string(),
    weight_preset: z.string(),
    data_freshness: z.string(),
    disclaimer: z.string(),
  }),
});
export type TlsResponse = z.infer<typeof TlsResponse>;

export type SubScoreDto = z.infer<typeof SubScoreDto>;
