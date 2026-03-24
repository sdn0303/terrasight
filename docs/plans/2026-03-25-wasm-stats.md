# Phase 2b: WASM Stats Local Aggregation

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add stats computation to the WASM SpatialEngine, replacing `/api/stats` for risk ratios, facility counts, and zoning distribution with client-side calculation using the existing R-tree spatial index.

**Architecture:** Extend `SpatialEngine` with `compute_stats()` method. Add `geo` crate for polygon area/intersection. Extract stats-specific columns at FGB load time into `LayerStatsData` enum. Worker gets new "compute-stats" message type. `useStats` hook becomes dual-path (WASM primary, API fallback). Land price stats computed from `useLandPrices` cache data in JS.

**Tech Stack:** Rust (geo 0.29, geozero, rstar), TypeScript (TanStack Query dual-path)

---

## Task 1: Rust — add `geo` dependency + stats data model

**Files:**
- Modify: `services/wasm/Cargo.toml`
- Create: `services/wasm/src/stats.rs`
- Modify: `services/wasm/src/spatial_index.rs` (add `LayerStatsData`)
- Modify: `services/wasm/src/fgb_reader.rs` (extract geometry + properties)
- Modify: `services/wasm/src/lib.rs` (add mod stats)

**Step 1: Add geo to Cargo.toml**

Add `geo = "0.29"` to dependencies.

**Step 2: Extend ParsedFeature**

In `fgb_reader.rs`, add to `ParsedFeature`:
```rust
pub geometry_geo: Option<geo::Geometry<f64>>,  // for area calculations
pub properties: Option<serde_json::Map<String, serde_json::Value>>,
```

During `parse_fgb`, extract:
- `geometry_geo`: use `geozero::geo_types::GeoWriter` to convert FGB geometry → `geo::Geometry`
- `properties`: parse the `properties` object from the GeoJSON string via `serde_json`

**Step 3: Add LayerStatsData to spatial_index.rs**

```rust
pub enum LayerStatsData {
    None,
    PricePoints(Vec<f64>),                          // land_prices: price_per_sqm
    AreaPolygons(Vec<geo::Geometry<f64>>),           // flood, steep_slope
    ZoningPolygons(Vec<(String, geo::Geometry<f64>)>), // zoning: (zone_type, geom)
    PointCount,                                      // schools, medical
}
```

Add `stats_data: LayerStatsData` field to `LayerIndex`.

In `from_parsed`, call `extract_stats_data(layer_id, &features)` to populate based on layer name:
- `"flood"` | `"flood-history"` | `"steep_slope"` → `AreaPolygons`
- `"zoning"` → `ZoningPolygons` (extract `zone_type` from properties)
- `"schools"` | `"medical"` → `PointCount`
- `"landprice"` → `PricePoints` (extract `price_per_sqm`)
- others → `None`

**Step 4: Create stats.rs**

```rust
use geo::prelude::*;
use geo::{Rect, Geometry, BooleanOps};

pub struct StatsResult {
    pub land_price: LandPriceStats,
    pub risk: RiskStats,
    pub facilities: FacilityStats,
    pub zoning_distribution: Vec<(String, f64)>,
}

pub struct LandPriceStats {
    pub avg_per_sqm: f64,
    pub median_per_sqm: f64,
    pub min_per_sqm: f64,
    pub max_per_sqm: f64,
    pub count: u32,
}

pub struct RiskStats {
    pub flood_area_ratio: f64,
    pub steep_slope_area_ratio: f64,
    pub composite_risk: f64,
}

pub struct FacilityStats {
    pub schools: u32,
    pub medical: u32,
}

// Risk weight constants (match backend domain/constants.rs)
const RISK_WEIGHT_FLOOD: f64 = 0.6;
const RISK_WEIGHT_STEEP: f64 = 0.4;
```

Implement functions:
- `compute_land_price_stats(index: &LayerIndex, indices: &[u32]) -> LandPriceStats`
  - Extract prices from `LayerStatsData::PricePoints` at matching indices
  - Sort, compute avg/median/min/max/count
- `compute_risk_stats(bbox_rect: Rect, flood_index: &LayerIndex, flood_hits: &[u32], steep_index: &LayerIndex, steep_hits: &[u32]) -> RiskStats`
  - For each polygon in hits: `polygon.intersection(&bbox_rect)` → `intersected.unsigned_area()`
  - Sum areas, divide by `bbox_rect.unsigned_area()`
  - composite = flood_ratio × 0.6 + steep_ratio × 0.4
- `compute_facility_count(hits: &[u32]) -> u32` — just `hits.len() as u32`
- `compute_zoning_distribution(bbox_rect: Rect, index: &LayerIndex, hits: &[u32]) -> Vec<(String, f64)>`
  - For each hit: extract (zone_type, geometry), intersect with bbox_rect, compute area
  - Group by zone_type, normalize to ratios, sort descending

**Step 5: cargo test**

Test with real FGB data: `data/fgb/13/geology.fgb` (available for basic tests), or create specific tests that load flood/zoning data if available.

```bash
cd services/wasm && cargo test
```

**Step 6: Commit**

```bash
git add services/wasm/
git commit -m "feat(wasm): add stats data model, geo dependency, and aggregation functions"
```

---

## Task 2: Rust — SpatialEngine.compute_stats() wasm_bindgen export

**Files:**
- Modify: `services/wasm/src/lib.rs`

**Step 1: Add compute_stats method**

```rust
#[wasm_bindgen]
impl SpatialEngine {
    /// Compute area stats. Returns JSON matching backend StatsResponse shape.
    /// land_price stats use indices from the "landprice" layer if loaded.
    /// risk stats use "flood-history" and "steep_slope" (or similar) layers.
    /// facility counts use "schools" and "medical" layers.
    /// zoning uses "zoning" layer.
    pub fn compute_stats(
        &self, south: f64, west: f64, north: f64, east: f64,
    ) -> Result<String, JsValue> {
        self.compute_stats_inner(south, west, north, east)
            .map_err(|e| JsValue::from_str(&e))
    }
}
```

`compute_stats_inner` logic:
1. Build `geo::Rect` from bbox
2. Query each relevant layer's R-tree for bbox hits
3. Call stats functions from `stats.rs`
4. Build JSON response matching `StatsResponse` schema:

```json
{
  "land_price": { "avg_per_sqm": 0, "median_per_sqm": 0, "min_per_sqm": 0, "max_per_sqm": 0, "count": 0 },
  "risk": { "flood_area_ratio": 0.15, "steep_slope_area_ratio": 0.02, "composite_risk": 0.10 },
  "facilities": { "schools": 12, "medical": 28 },
  "zoning_distribution": { "商業地域": 0.35 }
}
```

Layer name mapping (handle both dashes and underscores):
- land prices: try "landprice" layer
- flood: try "flood-history" or "flood"
- steep slope: try "steep-slope" or "steep_slope"
- schools: try "schools"
- medical: try "medical"
- zoning: try "zoning"

If a layer is not loaded, return zero/empty values for that section (graceful degradation).

**Step 2: cargo test + WASM build**

```bash
cd services/wasm && cargo test
bash scripts/build-wasm.sh
```

**Step 3: Commit**

```bash
git add services/wasm/
git commit -m "feat(wasm): add compute_stats wasm_bindgen export"
```

---

## Task 3: Frontend — Worker + Adapter + Type declarations

**Files:**
- Modify: `services/frontend/src/lib/wasm/wasm.d.ts`
- Modify: `services/frontend/src/lib/wasm/worker.ts`
- Modify: `services/frontend/src/lib/wasm/spatial-engine.ts`

**Step 1: Update wasm.d.ts**

Add to `ISpatialEngine` interface:
```typescript
compute_stats(south: number, west: number, north: number, east: number): string;
```

Add to `declare module` SpatialEngine class:
```typescript
compute_stats(south: number, west: number, north: number, east: number): string;
```

**Step 2: Update worker.ts**

Add message type:
```typescript
interface ComputeStatsMsg {
  type: "compute-stats";
  id: number;
  bbox: { south: number; west: number; north: number; east: number };
}
```

Add handler:
```typescript
case "compute-stats": {
  if (!engine) { send({ type: "error", message: "not initialized" }); break; }
  const json = engine.compute_stats(msg.bbox.south, msg.bbox.west, msg.bbox.north, msg.bbox.east);
  send({ type: "stats-result", id: msg.id, stats: json });
  break;
}
```

Add outgoing message type:
```typescript
interface StatsResultMsg { type: "stats-result"; id: number; stats: string; }
```

**Step 3: Update spatial-engine.ts**

Add method:
```typescript
async computeStats(bbox: BBox): Promise<StatsResponse> {
  if (!this.worker || !this._ready) throw new Error("not ready");
  const id = this.nextId++;
  return new Promise((resolve, reject) => {
    this.pending.set(id, {
      resolve: (data: string) => resolve(JSON.parse(data)),
      reject,
    });
    this.worker!.postMessage({ type: "compute-stats", id, bbox });
  });
}
```

Update `handleMessage` to handle `stats-result`:
```typescript
case "stats-result": {
  const p = this.pending.get(msg.id);
  if (p) { this.pending.delete(msg.id); p.resolve(msg.stats); }
  break;
}
```

**Step 4: tsc**

```bash
pnpm tsc --noEmit
```

**Step 5: Commit**

```bash
git add src/lib/wasm/
git commit -m "feat(frontend): add compute_stats to Worker and SpatialEngine adapter"
```

---

## Task 4: Frontend — useStats dual-path hook + land price JS stats

**Files:**
- Modify: `services/frontend/src/features/stats/api/use-stats.ts`
- Create: `services/frontend/src/features/stats/utils/compute-land-price-stats.ts`

**Step 1: Create land price stats utility**

JS function that computes avg/median/min/max/count from the `useLandPrices` response data (which is already cached by TanStack Query):

```typescript
import type { FeatureCollection } from "geojson";

export interface LandPriceStats {
  avg_per_sqm: number;
  median_per_sqm: number;
  min_per_sqm: number;
  max_per_sqm: number;
  count: number;
}

export function computeLandPriceStats(fc: FeatureCollection | undefined): LandPriceStats {
  if (!fc || fc.features.length === 0) {
    return { avg_per_sqm: 0, median_per_sqm: 0, min_per_sqm: 0, max_per_sqm: 0, count: 0 };
  }
  const prices = fc.features
    .map(f => (f.properties as Record<string, unknown>)?.price_per_sqm)
    .filter((p): p is number => typeof p === "number" && p > 0)
    .sort((a, b) => a - b);

  if (prices.length === 0) {
    return { avg_per_sqm: 0, median_per_sqm: 0, min_per_sqm: 0, max_per_sqm: 0, count: 0 };
  }

  const sum = prices.reduce((s, p) => s + p, 0);
  const mid = Math.floor(prices.length / 2);
  const median = prices.length % 2 === 0
    ? (prices[mid - 1]! + prices[mid]!) / 2
    : prices[mid]!;

  return {
    avg_per_sqm: Math.round(sum / prices.length),
    median_per_sqm: Math.round(median),
    min_per_sqm: prices[0]!,
    max_per_sqm: prices[prices.length - 1]!,
    count: prices.length,
  };
}
```

**Step 2: Update useStats — dual path**

```typescript
export function useStats(bbox: BBox | null, zoom: number) {
  const wasmReady = useSpatialEngineReady();
  const bboxKey = bbox ? `${bbox.south},${bbox.west},${bbox.north},${bbox.east}` : "";

  // WASM path
  const wasmResult = useQuery({
    queryKey: ["stats-wasm", bboxKey],
    queryFn: async () => {
      if (!bbox) throw new Error("bbox required");
      return spatialEngine.computeStats(bbox);
    },
    enabled: bbox !== null && zoom >= 10 && wasmReady,
    staleTime: 5_000,
  });

  // API fallback
  const apiResult = useQuery({
    queryKey: queryKeys.stats.bbox(bbox ?? { south: 0, west: 0, north: 0, east: 0 }),
    queryFn: ({ signal }) => {
      if (!bbox) throw new Error("bbox required");
      return fetchStats(bbox, signal);
    },
    enabled: bbox !== null && zoom >= 10 && !wasmReady,
    staleTime: 60_000,
    retry: 1,
  });

  return wasmReady ? wasmResult : apiResult;
}
```

**Step 3: tsc + vitest**

```bash
pnpm tsc --noEmit && pnpm vitest run
```

**Step 4: Commit**

```bash
git add src/features/stats/
git commit -m "feat(frontend): dual-path useStats with WASM primary and API fallback"
```

---

## Task 5: Integration test — full pipeline

**Step 1: Rust tests**

```bash
cd services/wasm && cargo test
```

**Step 2: WASM build**

```bash
bash scripts/build-wasm.sh
```

**Step 3: Frontend tests**

```bash
cd services/frontend && pnpm tsc --noEmit && pnpm vitest run
```

**Step 4: Docker build + run**

```bash
docker compose build frontend && docker compose up -d
```

**Step 5: Browser verification**

Open http://localhost:3001:
- DashboardStats should show stats (RISK, FACILITIES)
- Check DevTools Network: `/api/stats` should NOT be called if WASM is ready
- Check Console: no WASM errors
- Pan/zoom: stats should update immediately (< 5ms, no network round trip)

**Step 6: Commit any fixups**

```bash
git commit -m "fix: Phase 2b integration fixups"
```
