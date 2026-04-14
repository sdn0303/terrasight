import { z } from "zod";

const ManifestLayerSchema = z.object({
  id: z.string(),
  path: z.string(),
  features: z.number(),
  size_bytes: z.number(),
});

export const ManifestSchema = z.object({
  version: z.string(),
  prefectures: z.record(
    z.string(),
    z.object({ layers: z.array(ManifestLayerSchema) }),
  ),
});

export type Manifest = z.infer<typeof ManifestSchema>;
export type ManifestLayer = z.infer<typeof ManifestLayerSchema>;
