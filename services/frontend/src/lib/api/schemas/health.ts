import { z } from "zod";

export const HealthResponse = z.object({
  status: z.enum(["ok", "degraded"]),
  db_connected: z.boolean(),
  reinfolib_key_set: z.boolean(),
  version: z.string(),
});
export type HealthResponse = z.infer<typeof HealthResponse>;
