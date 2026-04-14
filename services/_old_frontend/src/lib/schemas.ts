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

/** LayerResponseDto — wraps FeatureCollection with server-side truncation metadata. */
function layerResponse<P extends z.ZodTypeAny>(properties: P) {
  return featureCollection(properties).extend({
    truncated: z.boolean(),
    count: z.number().int().nonnegative(),
    limit: z.number().int().nonnegative(),
  });
}

// ─── Layer property schemas ───────────────────────────
export const LandPriceProperties = z.object({
  id: z.number(),
  price_per_sqm: z.number(),
  address: z.string(),
  land_use: z.string().nullable(),
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

// ─── Area data response ───────────────────────────────
export const AreaDataResponse = z.object({
  landprice: layerResponse(LandPriceProperties).optional(),
  zoning: layerResponse(ZoningProperties).optional(),
  flood: layerResponse(FloodProperties).optional(),
  steep_slope: layerResponse(SteepSlopeProperties).optional(),
  schools: layerResponse(SchoolProperties).optional(),
  medical: layerResponse(MedicalProperties).optional(),
});

// ─── Score response ───────────────────────────────────
const SubScoreDto = z.object({
  id: z.string(),
  score: z.number(),
  available: z.boolean(),
  detail: z.record(z.string(), z.unknown()),
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
    composite_risk: z.number(),
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

// ─── Land price time-series response ─────────────────
export const LandPriceTimeSeriesResponse = layerResponse(LandPriceProperties);
export type LandPriceTimeSeriesResponse = z.infer<
  typeof LandPriceTimeSeriesResponse
>;

// ─── Area stats response ──────────────────────────────
export const AreaStatsResponse = z.object({
  code: z.string(),
  name: z.string(),
  level: z.enum(["prefecture", "municipality"]),
  land_price: z.object({
    avg_per_sqm: z.number().nullable(),
    median_per_sqm: z.number().nullable(),
    count: z.number(),
  }),
  risk: z.object({
    flood_area_ratio: z.number(),
    composite_risk: z.number(),
  }),
  facilities: z.object({
    schools: z.number(),
    medical: z.number(),
  }),
});

// ─── Opportunities response (Phase 5) ────────────────
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

// ─── Export inferred types ────────────────────────────
export type AreaDataResponse = z.infer<typeof AreaDataResponse>;
export type TlsResponse = z.infer<typeof TlsResponse>;
export type StatsResponse = z.infer<typeof StatsResponse>;
export type TrendResponse = z.infer<typeof TrendResponse>;
export type HealthResponse = z.infer<typeof HealthResponse>;
export type LandPriceProperties = z.infer<typeof LandPriceProperties>;
export type ZoningProperties = z.infer<typeof ZoningProperties>;
export type AreaStatsResponse = z.infer<typeof AreaStatsResponse>;
