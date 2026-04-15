import { z } from "zod";

// ─── GeoJSON primitives ───────────────────────────────
export const PointGeometry = z.object({
  type: z.literal("Point"),
  coordinates: z.tuple([z.number(), z.number()]), // [lng, lat] per RFC 7946
});

export const PolygonGeometry = z.object({
  type: z.literal("Polygon"),
  coordinates: z.array(z.array(z.tuple([z.number(), z.number()]))),
});

export const MultiPolygonGeometry = z.object({
  type: z.literal("MultiPolygon"),
  coordinates: z.array(z.array(z.array(z.tuple([z.number(), z.number()])))),
});

export const Geometry = z.discriminatedUnion("type", [
  PointGeometry,
  PolygonGeometry,
  MultiPolygonGeometry,
]);

export function featureCollection<P extends z.ZodTypeAny>(properties: P) {
  return z.object({
    type: z.literal("FeatureCollection"),
    features: z.array(
      z.object({
        type: z.literal("Feature"),
        geometry: Geometry,
        properties: properties,
      }),
    ),
  });
}

/** LayerResponseDto — wraps FeatureCollection with server-side truncation metadata. */
export function layerResponse<P extends z.ZodTypeAny>(properties: P) {
  return featureCollection(properties).extend({
    truncated: z.boolean(),
    count: z.number().int().nonnegative(),
    limit: z.number().int().nonnegative(),
  });
}
