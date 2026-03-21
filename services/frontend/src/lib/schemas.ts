import { z } from "zod";

// ─── GeoJSON primitives ───────────────────────────────
const PointGeometry = z.object({
  type: z.literal("Point"),
  coordinates: z.tuple([z.number(), z.number()]), // [lng, lat] per RFC 7946
});

const PolygonGeometry = z.object({
  type: z.literal("Polygon"),
  coordinates: z.array(z.array(z.tuple([z.number(), z.number()]))),
});

const MultiPolygonGeometry = z.object({
  type: z.literal("MultiPolygon"),
  coordinates: z.array(z.array(z.array(z.tuple([z.number(), z.number()])))),
});

const Geometry = z.discriminatedUnion("type", [
  PointGeometry,
  PolygonGeometry,
  MultiPolygonGeometry,
]);

function featureCollection<P extends z.ZodTypeAny>(properties: P) {
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

// ─── Layer property schemas ───────────────────────────
export const LandPriceProperties = z.object({
  id: z.number(),
  price_per_sqm: z.number(),
  address: z.string(),
  land_use: z.string(),
  year: z.number(),
});

export const ZoningProperties = z.object({
  id: z.number(),
  zone_type: z.string(),
  zone_code: z.string(),
  floor_area_ratio: z.number(),
  building_coverage: z.number(),
});

export const FloodProperties = z.object({
  id: z.number(),
  depth_rank: z.number(),
  river_name: z.string(),
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

// ─── Area data response ───────────────────────────────
export const AreaDataResponse = z.object({
  landprice: featureCollection(LandPriceProperties).optional(),
  zoning: featureCollection(ZoningProperties).optional(),
  flood: featureCollection(FloodProperties).optional(),
  steep_slope: featureCollection(SteepSlopeProperties).optional(),
  schools: featureCollection(SchoolProperties).optional(),
  medical: featureCollection(MedicalProperties).optional(),
});

// ─── Score response ───────────────────────────────────
const ScoreComponent = z.object({
  value: z.number(),
  max: z.number(),
  detail: z.record(z.string(), z.unknown()),
});

export const ScoreResponse = z.object({
  score: z.number(),
  components: z.object({
    trend: ScoreComponent,
    risk: ScoreComponent,
    access: ScoreComponent,
    yield_potential: ScoreComponent,
  }),
  metadata: z.object({
    calculated_at: z.string(),
    data_freshness: z.string(),
    disclaimer: z.string(),
  }),
});

// ─── Stats response ───────────────────────────────────
export const StatsResponse = z.object({
  land_price: z.object({
    avg_per_sqm: z.number(),
    median_per_sqm: z.number(),
    min_per_sqm: z.number(),
    max_per_sqm: z.number(),
    count: z.number(),
  }),
  risk: z.object({
    flood_area_ratio: z.number(),
    steep_slope_area_ratio: z.number(),
    avg_composite_risk: z.number(),
  }),
  facilities: z.object({
    schools: z.number(),
    medical: z.number(),
  }),
  zoning_distribution: z.record(z.string(), z.number()),
});

// ─── Trend response ───────────────────────────────────
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

// ─── Health response ──────────────────────────────────
export const HealthResponse = z.object({
  status: z.enum(["ok", "degraded"]),
  db_connected: z.boolean(),
  reinfolib_key_set: z.boolean(),
  version: z.string(),
});

// ─── Export inferred types ────────────────────────────
export type AreaDataResponse = z.infer<typeof AreaDataResponse>;
export type ScoreResponse = z.infer<typeof ScoreResponse>;
export type StatsResponse = z.infer<typeof StatsResponse>;
export type TrendResponse = z.infer<typeof TrendResponse>;
export type HealthResponse = z.infer<typeof HealthResponse>;
export type LandPriceProperties = z.infer<typeof LandPriceProperties>;
export type ZoningProperties = z.infer<typeof ZoningProperties>;
