# Phase 1A: API Safety Limits Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add zoom-aware, per-layer dynamic LIMIT to all bbox queries, Polygon conversion for land prices, and truncation UX to prepare for nationwide 47-prefecture scale.

**Architecture:** Backend computes `LIMIT = min(bbox_area × layer_density, 10_000)` with zoom factor. Fetches N+1 rows to detect truncation without COUNT(*). Frontend sends zoom, displays per-layer truncation info in StatusBar.

**Tech Stack:** Rust (Axum, SQLx, PostGIS), TypeScript (Next.js 16, TanStack Query, Zod, react-map-gl)

---

## Task 1: Backend — density constants + limit function

**Files:**
- Modify: `services/backend/src/domain/constants.rs` (append)
- Modify: `services/backend/lib/geo-math/src/spatial.rs` (append)
- Test: `services/backend/lib/geo-math/src/spatial.rs` (inline tests)

**Step 1: Write failing test for `compute_feature_limit`**

Add to `services/backend/lib/geo-math/src/spatial.rs`:

```rust
#[cfg(test)]
mod tests {
    // ... existing tests ...

    #[test]
    fn feature_limit_small_bbox_flood() {
        // Tokyo 1 ward: 0.02 deg² × 150_000 density = 3_000
        assert_eq!(super::compute_feature_limit("flood", 0.02, 12), 3_000);
    }

    #[test]
    fn feature_limit_caps_at_max() {
        // Large bbox: 1.0 deg² × 150_000 = 150_000 → capped at 10_000
        assert_eq!(super::compute_feature_limit("flood", 1.0, 12), 10_000);
    }

    #[test]
    fn feature_limit_low_zoom_divides() {
        // zoom 8: 0.5 × 150_000 = 75_000 → cap 10_000 → ÷4 = 2_500
        assert_eq!(super::compute_feature_limit("flood", 0.5, 8), 2_500);
    }

    #[test]
    fn feature_limit_unknown_layer_uses_default() {
        assert!(super::compute_feature_limit("unknown", 0.01, 12) > 0);
    }

    #[test]
    fn point_to_polygon_creates_closed_ring() {
        let ring = super::point_to_polygon(139.7, 35.68);
        assert_eq!(ring.len(), 5);
        assert_eq!(ring[0], ring[4]); // closed ring
    }

    #[test]
    fn point_to_polygon_buffer_size() {
        let ring = super::point_to_polygon(139.7, 35.68);
        let width = ring[1][0] - ring[0][0];
        assert!((width - 2.0 * super::BUFFER_DEG).abs() < f64::EPSILON);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd services/backend && cargo test -p realestate-geo-math`
Expected: FAIL — `compute_feature_limit` and `point_to_polygon` not defined

**Step 3: Implement constants and functions**

Add to `services/backend/lib/geo-math/src/spatial.rs`:

```rust
/// Buffer offset in degrees for ~30m x 30m squares at Tokyo latitude (~35.68°).
pub const BUFFER_DEG: f64 = 0.00015;

/// Per-layer max feature density (features / deg²).
const LAYER_DENSITY: &[(&str, f64)] = &[
    ("landprice", 50_000.0),
    ("flood", 150_000.0),
    ("zoning", 20_000.0),
    ("steep_slope", 17_000.0),
    ("schools", 40_000.0),
    ("medical", 27_000.0),
];

/// Absolute ceiling per layer per request.
const MAX_FEATURES_PER_LAYER: i64 = 10_000;

const DEFAULT_DENSITY: f64 = 30_000.0;

/// Compute dynamic LIMIT based on bbox area, layer density, and zoom level.
pub fn compute_feature_limit(layer: &str, bbox_area_deg2: f64, zoom: u8) -> i64 {
    let density = LAYER_DENSITY
        .iter()
        .find(|(name, _)| *name == layer)
        .map(|(_, d)| *d)
        .unwrap_or(DEFAULT_DENSITY);

    let raw = (bbox_area_deg2 * density).ceil() as i64;
    let capped = raw.min(MAX_FEATURES_PER_LAYER).max(1);
    if zoom < 10 { capped / 4 } else { capped }
}

/// Convert a Point [lng, lat] to a closed Polygon ring (5 vertices, ~30m x 30m).
pub fn point_to_polygon(lng: f64, lat: f64) -> [[f64; 2]; 5] {
    [
        [lng - BUFFER_DEG, lat - BUFFER_DEG],
        [lng + BUFFER_DEG, lat - BUFFER_DEG],
        [lng + BUFFER_DEG, lat + BUFFER_DEG],
        [lng - BUFFER_DEG, lat + BUFFER_DEG],
        [lng - BUFFER_DEG, lat - BUFFER_DEG],
    ]
}
```

**Step 4: Run tests**

Run: `cd services/backend && cargo test -p realestate-geo-math`
Expected: All pass

**Step 5: Commit**

```bash
git add services/backend/lib/geo-math/src/spatial.rs
git commit -m "feat(geo-math): add compute_feature_limit and point_to_polygon"
```

---

## Task 2: Backend — AreaRepository trait + impl with LIMIT

**Files:**
- Modify: `services/backend/src/domain/repository.rs:13-21` (change trait signatures)
- Modify: `services/backend/src/infra/pg_area_repository.rs` (add LIMIT + truncated)

**Step 1: Define LayerResult return type**

Add to `services/backend/src/domain/entity.rs` (or a new `services/backend/src/domain/layer_result.rs`):

```rust
/// Result of a spatially-limited layer query.
pub struct LayerResult {
    pub features: Vec<GeoFeature>,
    pub truncated: bool,
    pub limit: i64,
}
```

**Step 2: Update AreaRepository trait**

Change `services/backend/src/domain/repository.rs` — all methods now take `zoom: u8` and return `LayerResult`:

```rust
#[async_trait]
pub trait AreaRepository: Send + Sync {
    async fn find_land_prices(&self, bbox: &BBox, zoom: u8) -> Result<LayerResult, DomainError>;
    async fn find_zoning(&self, bbox: &BBox, zoom: u8) -> Result<LayerResult, DomainError>;
    async fn find_flood_risk(&self, bbox: &BBox, zoom: u8) -> Result<LayerResult, DomainError>;
    async fn find_steep_slope(&self, bbox: &BBox, zoom: u8) -> Result<LayerResult, DomainError>;
    async fn find_schools(&self, bbox: &BBox, zoom: u8) -> Result<LayerResult, DomainError>;
    async fn find_medical(&self, bbox: &BBox, zoom: u8) -> Result<LayerResult, DomainError>;
}
```

**Step 3: Update PgAreaRepository impl**

For each `find_*` method in `pg_area_repository.rs`, apply this pattern:

```rust
async fn find_land_prices(&self, bbox: &BBox, zoom: u8) -> Result<LayerResult, DomainError> {
    let bbox_area = bbox_area_deg2(bbox.south(), bbox.west(), bbox.north(), bbox.east());
    let limit = compute_feature_limit("landprice", bbox_area, zoom);

    let query = sqlx::query_as::<_, (i64, i32, String, Option<String>, i32, serde_json::Value)>(
        r#"
        SELECT id, price_per_sqm, address, land_use, year,
               ST_AsGeoJSON(geom)::jsonb AS geometry
        FROM land_prices
        WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
        LIMIT $5
        "#,
    );
    let rows = bind_bbox(query, bbox.west(), bbox.south(), bbox.east(), bbox.north())
        .bind(limit + 1)  // fetch N+1 for truncation detection
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err)?;

    let truncated = rows.len() as i64 > limit;
    let features = if truncated {
        // ... map rows[..limit] to GeoFeature
    } else {
        // ... map all rows to GeoFeature
    };

    Ok(LayerResult { features, truncated, limit })
}
```

Repeat for all 6 methods with appropriate layer name strings.

**Step 4: Fix all callers** — update usecases that call AreaRepository to pass `zoom` through.

- `services/backend/src/usecase/get_area_data.rs` — accept `zoom: u8`, pass to repository
- `services/backend/src/handler/area_data.rs` — extract `zoom` from query params, pass to usecase

**Step 5: cargo clippy + cargo test**

Run: `cd services/backend && cargo clippy -- -D warnings && cargo test`
Expected: All pass

**Step 6: Commit**

```bash
git add services/backend/src/domain/ services/backend/src/infra/ services/backend/src/usecase/ services/backend/src/handler/
git commit -m "feat(backend): add zoom-aware per-layer LIMIT to all bbox queries"
```

---

## Task 3: Backend — Polygon conversion for land prices

**Files:**
- Modify: `services/backend/src/handler/area_data.rs:41-56` (apply polygon conversion)
- Modify: `services/backend/src/handler/land_price.rs:46-50` (apply polygon conversion)
- Modify: `services/backend/src/handler/response.rs` (add LayerResponseDto + polygon helper)

**Step 1: Add polygon conversion helper to response.rs**

```rust
use realestate_geo_math::spatial::point_to_polygon;

/// Convert a FeatureDto's Point geometry to a Polygon (30m square).
pub fn point_feature_to_polygon(mut feature: FeatureDto) -> FeatureDto {
    if let Some(geom) = &feature.geometry {
        if geom.r#type == "Point" {
            if let Some(coords) = geom.coordinates.as_array() {
                if let (Some(lng), Some(lat)) = (coords.first().and_then(|v| v.as_f64()), coords.get(1).and_then(|v| v.as_f64())) {
                    let ring = point_to_polygon(lng, lat);
                    feature.geometry = Some(GeometryDto {
                        r#type: "Polygon".to_string(),
                        coordinates: serde_json::json!([ring]),
                    });
                }
            }
        }
    }
    feature
}
```

**Step 2: Add LayerResponseDto**

```rust
#[derive(Serialize)]
pub struct LayerResponseDto {
    pub r#type: String,
    pub features: Vec<FeatureDto>,
    pub truncated: bool,
    pub count: usize,
    pub limit: i64,
}

impl LayerResponseDto {
    pub fn from_layer_result(result: LayerResult, polygon_convert: bool) -> Self {
        let features: Vec<FeatureDto> = result.features
            .into_iter()
            .map(geo_feature_to_dto)
            .map(|f| if polygon_convert { point_feature_to_polygon(f) } else { f })
            .collect();
        let count = features.len();
        Self {
            r#type: "FeatureCollection".to_string(),
            features,
            truncated: result.truncated,
            count,
            limit: result.limit,
        }
    }
}
```

**Step 3: Update area_data.rs handler** to use `LayerResponseDto` and apply polygon conversion for landprice layer.

**Step 4: Update land_price.rs handler** to use `LayerResponseDto` and apply polygon conversion.

**Step 5: cargo clippy + cargo test**

Run: `cd services/backend && cargo clippy -- -D warnings && cargo test`

**Step 6: Commit**

```bash
git add services/backend/src/handler/
git commit -m "feat(backend): convert landprice to Polygon and add truncation metadata"
```

---

## Task 4: Frontend — schema + API changes

**Files:**
- Modify: `services/frontend/src/lib/schemas.ts:80-87` (add truncated/count/limit)
- Modify: `services/frontend/src/lib/api.ts:107-124,152-166` (add zoom param)
- Modify: `services/frontend/src/features/area-data/api/use-area-data.ts` (add zoom, enabled guard)
- Modify: `services/frontend/src/features/stats/api/use-stats.ts` (add zoom guard)
- Modify: `services/frontend/src/features/land-prices/api/use-land-prices.ts` (add zoom to fetch)
- Modify: `services/frontend/src/app/page.tsx` (pass zoom to hooks)

**Step 1: Update Zod schemas**

In `schemas.ts`, create a wrapper for layer responses:

```typescript
const layerResponse = <T extends z.ZodTypeAny>(props: T) =>
  z.object({
    type: z.literal("FeatureCollection"),
    features: z.array(
      z.object({
        type: z.literal("Feature"),
        geometry: z.unknown(),
        properties: props,
      }),
    ),
    truncated: z.boolean(),
    count: z.number(),
    limit: z.number(),
  });

export const AreaDataResponse = z.object({
  landprice: layerResponse(LandPriceProperties).optional(),
  zoning: layerResponse(ZoningProperties).optional(),
  flood: layerResponse(FloodProperties).optional(),
  steep_slope: layerResponse(SteepSlopeProperties).optional(),
  schools: layerResponse(SchoolProperties).optional(),
  medical: layerResponse(MedicalProperties).optional(),
});
```

**Step 2: Update API functions** — add `zoom` parameter

```typescript
export function fetchAreaData(
  bbox: BBox,
  layers: string[],
  zoom: number,
  signal?: AbortSignal,
) {
  return get(AreaDataResponse, "api/area-data", {
    south: String(bbox.south),
    west: String(bbox.west),
    north: String(bbox.north),
    east: String(bbox.east),
    layers: layers.join(","),
    zoom: String(Math.floor(zoom)),
  }, signal);
}
```

Same for `fetchLandPrices` — add `zoom` param.

**Step 3: Update hooks**

`use-area-data.ts` — add zoom param + guard:
```typescript
export function useAreaData(bbox: BBox | null, layers: string[], zoom: number) {
  return useQuery({
    queryKey: queryKeys.areaData.bbox(bbox ?? { south: 0, west: 0, north: 0, east: 0 }, layers),
    queryFn: ({ signal }) => {
      if (bbox === null) throw new Error("bbox is required");
      return fetchAreaData(bbox, layers, zoom, signal);
    },
    enabled: bbox !== null && layers.length > 0 && zoom >= 10,
    staleTime: 60_000,
  });
}
```

`use-stats.ts` — add zoom guard:
```typescript
export function useStats(bbox: BBox | null, zoom: number) {
  return useQuery({
    ...
    enabled: bbox !== null && zoom >= 10,
    ...
  });
}
```

**Step 4: Update page.tsx** — pass `viewState.zoom` to all affected hooks.

**Step 5: tsc + vitest**

Run: `cd services/frontend && pnpm tsc --noEmit && pnpm vitest run`

**Step 6: Commit**

```bash
git add services/frontend/src/
git commit -m "feat(frontend): add zoom param to API calls and zoom >= 10 guard"
```

---

## Task 5: Frontend — remove pointsToPolygons + truncation UX

**Files:**
- Delete: `services/frontend/src/features/land-prices/utils/points-to-polygons.ts`
- Modify: `services/frontend/src/components/map/layers/land-price-extrusion-layer.tsx` (remove import + useMemo)
- Modify: `services/frontend/src/components/status-bar.tsx` (add truncation display)
- Modify: `services/frontend/src/app/page.tsx` (pass truncation info to StatusBar)

**Step 1: Remove pointsToPolygons**

Delete `services/frontend/src/features/land-prices/utils/points-to-polygons.ts`.

In `land-price-extrusion-layer.tsx`, remove:
```typescript
import { pointsToPolygons } from "@/features/land-prices/utils/points-to-polygons";
// ...
const polygonData = useMemo(() => pointsToPolygons(data), [data]);
```

Replace with direct use of `data`:
```typescript
// data is already Polygon from backend
if (!visible || data.features.length === 0) return null;
```

Replace all `polygonData` references with `data`.

**Step 2: Add truncation info to StatusBar**

```typescript
interface TruncationInfo {
  layer: string;
  count: number;
  limit: number;
}

interface StatusBarProps {
  lat: number;
  lng: number;
  zoom: number;
  isLoading: boolean;
  isDemoMode: boolean;
  truncatedLayers?: TruncationInfo[];
}
```

Add to StatusBar JSX:
```typescript
{truncatedLayers && truncatedLayers.length > 0 && (
  <span style={{ color: "var(--accent-warning)" }}>
    ⚠ {truncatedLayers.map(t => `${t.layer}: ${t.count}/${t.limit}`).join(", ")}
  </span>
)}
```

**Step 3: Wire truncation from page.tsx** — derive `truncatedLayers` from `areaData` response.

**Step 4: tsc + vitest**

Run: `cd services/frontend && pnpm tsc --noEmit && pnpm vitest run`

**Step 5: Commit**

```bash
git add -A
git commit -m "feat(frontend): remove pointsToPolygons, add truncation display"
```

---

## Task 6: Integration test + Docker build

**Step 1: Full backend test**

Run: `cd services/backend && cargo test && cargo clippy -- -D warnings`

**Step 2: Full frontend test**

Run: `cd services/frontend && pnpm tsc --noEmit && pnpm biome check . && pnpm vitest run`

**Step 3: Docker build**

Run: `docker compose build`

**Step 4: Browser verification**

Run: `docker compose up -d`
- Open http://localhost:3001
- Zoom to level 8 (wide view) → verify truncation warning in StatusBar
- Zoom to level 14 (street) → verify no truncation, all data visible
- Verify land price 3D columns render (Polygon from backend)

**Step 5: Final commit if any fixups needed**

```bash
git commit -m "fix: integration fixups for Phase 1A"
```
