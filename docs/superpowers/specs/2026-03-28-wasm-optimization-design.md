# WASM Optimization Design Spec

## Problem Statement

The WASM spatial engine has three systemic issues:
1. **Correctness**: `ready` is a single boolean — partial layer loads silently produce incorrect stats
2. **Boundary cost**: `query_layers` returns double-JSON (per-layer GeoJSON strings inside outer JSON), causing 2x parse overhead
3. **Query granularity**: Each static layer component fires individual `useStaticLayer()` calls, creating N worker roundtrips per viewport change

## Design Decisions

| Decision | Choice | Alternatives Considered |
|----------|--------|------------------------|
| Shared domain code | Workspace crate (`services/shared-domain/`) | Backend-as-lib (feature flag hell), Constants-only file (no parity guarantee) |
| Observability | Performance API (`performance.mark/measure`) | pino-only (no p95), Custom collector (unnecessary) |
| Layer loading | 2-stage boot + on-demand | Theme-driven only (UX lag), Viewport-driven only (memory) |
| shared-domain scope | Constants + pure computation only | Include DTO shapes (rejected: Zod is API contract source of truth per CLAUDE.md) |

---

## Prerequisite: Canonical Layer ID Table

Layer IDs are inconsistent across systems. This MUST be resolved before any Phase work.

**Current state:**

| UI ID (layers.ts) | WASM ID (manifest) | FGB filename | Source | Conflict |
|---|---|---|---|---|
| `admin_boundary` | `admin-boundary` | `admin-boundary.fgb` | static | underscore vs hyphen |
| `flood_history` | `flood-history` | `flood-history.fgb` | static | underscore vs hyphen |
| `steep_slope` | *(not in WASM)* | *(no FGB)* | api | API-only, no WASM |
| `land_price_ts` | *(not in WASM)* | *(no FGB)* | timeseries | API-only, no WASM |
| `population_mesh` | *(not in WASM)* | `population-mesh.fgb` | static | not in manifest + naming |
| `landprice` | *(not in WASM)* | *(no FGB)* | api | API-only, no WASM |
| `schools` | *(not in WASM)* | *(no FGB)* | api | API-only, no WASM |
| `medical` | *(not in WASM)* | *(no FGB)* | api | API-only, no WASM |
| `zoning` | *(not in WASM)* | *(no FGB)* | api | API-only, no WASM |

**Resolution: Normalize to hyphen-case everywhere.**

Add a `canonicalId` mapping to `layers.ts`:

```typescript
// lib/layer-ids.ts — single source of truth for ID normalization
const ID_NORMALIZE: Record<string, string> = {
  admin_boundary: "admin-boundary",
  flood_history: "flood-history",
  steep_slope: "steep-slope",
  land_price_ts: "land-price-ts",
  population_mesh: "population-mesh",
};

/** Convert any layer ID variant to canonical hyphen-case form. */
export function canonicalLayerId(id: string): string {
  return ID_NORMALIZE[id] ?? id;
}
```

- WASM manifest, FGB filenames, and `query_layers` all use canonical (hyphen-case) IDs
- UI code (`layers.ts`, `themes.ts`, stores) continues using underscore IDs for backwards compatibility
- `canonicalLayerId()` is called at every WASM/FGB boundary
- `queryReady()`, `getBootLayers()`, and `loadedLayers` all operate on canonical IDs

**Impact:** This is Task 0 — implemented before Phase 1.

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│ Frontend (React 19)                                      │
│                                                          │
│  ┌──────────────────┐    ┌───────────────────────────┐  │
│  │ SpatialEngine    │    │ useVisibleStaticLayers()   │  │
│  │ Provider         │◄───│ single hook, batched query │  │
│  │ (app-level       │    │ + FGB asset cache (sep.)   │  │
│  │  singleton)      │    └───────────────────────────┘  │
│  └────────┬─────────┘                                    │
│           │              ┌───────────────────────────┐  │
│           │              │ Performance API metrics    │  │
│           │              │ wasm-init, layer-load,     │  │
│           │              │ wasm-query                 │  │
│           ▼              └───────────────────────────┘  │
│  ┌──────────────────┐                                    │
│  │ Web Worker        │                                   │
│  │ ┌──────────────┐ │                                   │
│  │ │ WASM Module  │ │                                   │
│  │ │ SpatialEngine│ │                                   │
│  │ │ (R-tree/layer│ │                                   │
│  │ │  per FGB)    │ │                                   │
│  │ └──────┬───────┘ │                                   │
│  │        │         │                                   │
│  │   shared-domain  │                                   │
│  │   (consts+calc)  │                                   │
│  └──────────────────┘                                    │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ Backend (Rust Axum)                                      │
│                                                          │
│  handler → usecase → domain ← infra                     │
│                        │                                 │
│                   shared-domain                          │
│                   (consts+calc, same crate as WASM)      │
└─────────────────────────────────────────────────────────┘
```

---

## Phase 1: Correctness

### 1.1 Ready State Separation

**Current**: `_ready: boolean` — set to `true` on any `init-done`, even with partial loads.

**New**: Replace with `loadedLayers: Set<string>` derived from `init-done` counts. All IDs stored in canonical (hyphen-case) form.

```typescript
// spatial-engine.ts
class SpatialEngineAdapter {
  private loadedLayers = new Set<string>(); // canonical IDs

  /** Check if ALL specified layers are loaded (accepts any ID form). */
  queryReady(layerIds: string[]): boolean {
    return layerIds.every(id => this.loadedLayers.has(canonicalLayerId(id)));
  }

  /**
   * Stats readiness — only true when ALL layers used by compute_stats are loaded.
   *
   * IMPORTANT: These are the static FGB layers that the WASM R-tree needs.
   * API-only layers (landprice, schools, medical, zoning, steep-slope) are
   * NOT loaded via FGB — they come from /api/area-data at query time.
   * For WASM stats to work, API data must be ingested into the R-tree
   * separately (see Phase 3: stats data ingestion).
   */
  get statsReady(): boolean {
    // Phase 1: Only static layers that exist in FGB
    const STATS_STATIC_LAYERS = ["flood-history"];
    // Phase 3 will expand this when API data ingestion is implemented
    return STATS_STATIC_LAYERS.every(id => this.loadedLayers.has(id));
  }
}
```

**Stats readiness reality check:**

The current WASM `compute_stats()` expects these layers in its R-tree:
- `landprice` — **API-only** (PostGIS L01 data, not in FGB)
- `flood-history` — **static FGB** (available)
- `steep-slope` / `steep_slope` — **API-only** (PostGIS A47 data)
- `schools` — **API-only** (PostGIS P29 data)
- `medical` — **API-only** (PostGIS P04 data)
- `zoning` — **API-only** (PostGIS A29 data)

**Conclusion:** Full WASM stats parity is NOT achievable with static FGB data alone. The WASM stats path computes incorrect values because 5 of 6 required datasets are API-only.

**Phased approach to stats:**
1. **Phase 1**: `statsReady` reflects reality — only `flood-history` can be validated. WASM stats stays disabled. Backend `/api/stats` remains canonical.
2. **Phase 3**: Consider ingesting API layer data into WASM R-tree at query time (pass FeatureCollection from `useAreaData` response into worker). This is a new `ingest-features` message, NOT a FGB load. Deferred to Phase 3 design.

### 1.2 Request-Scoped Error Handling

**Current problem:** Worker `error` message has no request `id`. Adapter rejects ALL pending queries on any error.

**Fix:** Add typed error messages with request IDs:

```typescript
// Worker → Main message protocol (complete)
type OutgoingMessage =
  | { type: "init-done"; counts: Record<string, number> }
  | { type: "query-result"; id: number; geojson: string }
  | { type: "query-error"; id: number; error: string }    // NEW
  | { type: "stats-result"; id: number; stats: string }
  | { type: "stats-error"; id: number; error: string }    // NEW
  | { type: "layer-loaded"; id: string; count: number }   // Phase 3
  | { type: "layer-load-failed"; id: string; error: string } // Phase 3
  | { type: "error"; message: string };  // KEPT: init-level errors only
```

Worker changes:
```typescript
// Before: catch-all error
try {
  const geojson = engine.query_layers(...);
  send({ type: "query-result", id, geojson });
} catch (err) {
  send({ type: "error", message: `query_layers failed: ${message}` });
  // ^^^ no id, adapter rejects ALL pending
}

// After: request-scoped error
try {
  const geojson = engine.query_layers(...);
  send({ type: "query-result", id, geojson });
} catch (err) {
  const message = err instanceof Error ? err.message : String(err);
  send({ type: "query-error", id, error: `query_layers failed: ${message}` });
  // ^^^ includes id, adapter rejects ONLY this request
}
```

Adapter changes:
```typescript
case "query-error": {
  const pending = this.pending.get(msg.id);
  if (pending) {
    this.pending.delete(msg.id);
    pending.reject(new Error(msg.error));
  }
  break;
}
```

The catch-all `"error"` type is retained for init-level failures only (no `id` available).

### 1.3 Stats Gating

- WASM stats path stays disabled (current state from X-02 fix)
- `statsReady` accurately reflects which static layers are loaded
- Full stats parity requires API data ingestion (Phase 3)
- Canonical source remains `/api/stats`
- Parity test (Phase 3): vitest that calls both paths with same bbox, asserts delta < 0.01

### 1.4 Observability (Performance API)

```typescript
// spatial-engine.ts — init timing
performance.mark("wasm-init-start");
// ... init ...
performance.mark("wasm-init-done");
performance.measure("wasm-init", "wasm-init-start", "wasm-init-done");

// worker.ts — per-layer load timing
performance.mark(`layer-load-${id}-start`);
// ... load_layer ...
performance.mark(`layer-load-${id}-done`);
performance.measure(`layer-load-${id}`, `layer-load-${id}-start`, `layer-load-${id}-done`);

// spatial-engine.ts — per-query timing
performance.mark(`wasm-query-${queryId}-start`);
// ... query_layers ...
performance.mark(`wasm-query-${queryId}-done`);
performance.measure(`wasm-query-${queryId}`, `wasm-query-${queryId}-start`, `wasm-query-${queryId}-done`);
```

Summary logged via pino: `log.info({ wasm_init_ms, loaded_count, failed_layers })`.

---

## Phase 2: Boundary Optimization

### 2.1 Batched Static Layer Query

**Current flow** (N roundtrips per viewport):
```
geology-layer.tsx  → useStaticLayer("13","geology") → worker query → parse → render
landform-layer.tsx → useStaticLayer("13","landform") → worker query → parse → render
station-layer.tsx  → useStaticLayer("13","station") → worker query → parse → render
... (11+ layers)
```

**New flow** (1 roundtrip per viewport):
```
useVisibleStaticLayers(bbox, visibleLayerIds)
  → single worker query_layers("geology,landform,station", bbox)
  → single JSON.parse
  → Map<string, FeatureCollection>
  → each layer component receives data as prop
```

**Hook signature:**
```typescript
function useVisibleStaticLayers(
  bbox: BBox | null,
  visibleLayerIds: string[],
): {
  data: Map<string, FeatureCollection>;
  isLoading: boolean;
  loadedLayers: Set<string>;
}
```

**Component change**: Static layer components switch from self-fetching to prop-receiving:
```typescript
// Before
export function GeologyLayer({ visible }: Props) {
  const { data } = useStaticLayer("13", "geology", visible);
  ...
}

// After
export function GeologyLayer({ visible, data }: Props) {
  if (!visible || !data) return null;
  return <Source data={data}>...</Source>;
}
```

**useStaticLayer** is deprecated and eventually removed.

### 2.2 JSON Flattening (Rust side)

**Current** `query_layers_inner`:
```rust
let mut result: HashMap<&str, String> = HashMap::new();
for layer_id in ... {
    result.insert(layer_id, index.get_features_geojson(&indices));
    // ^^ String value = serialized FeatureCollection
}
serde_json::to_string(&result) // Double serialization!
```

**New**:
```rust
let mut result: HashMap<&str, serde_json::Value> = HashMap::new();
for layer_id in ... {
    let fc = index.get_features_as_value(&indices);
    // ^^ serde_json::Value, not String
    result.insert(layer_id, fc);
}
serde_json::to_string(&result) // Single serialization
```

JS side: `JSON.parse(msg.geojson)` → directly get `{ geology: {type:"FeatureCollection",...}, ... }`. No inner parse loop.

### 2.3 Fallback Path — Two-Layer Cache Architecture

**Problem identified by Codex review:** The current fallback uses bbox-independent `staleTime: Infinity` cache for full FGB loads. Merging this into the bbox-dependent batch hook would break the cache semantics and cause re-fetches on every pan.

**Solution: Separate the two cache layers.**

```
Layer 1: FGB Asset Cache (bbox-independent, infinite TTL)
  queryKey: ["fgb-asset", prefCode, layerId]
  queryFn: fetch + flatgeobuf/deserialize → FeatureCollection
  staleTime: Infinity
  gcTime: Infinity
  Purpose: Cache the full FGB dataset. Fetched ONCE, never refetched.

Layer 2: Viewport Query Cache (bbox-dependent, short TTL)
  queryKey: ["static-layers-viewport", bbox.south, bbox.west, ..., sortedLayerIds]
  queryFn: engine.query(bbox, wasmReadyLayers) → Map<string, FeatureCollection>
  staleTime: 5_000
  Purpose: WASM R-tree viewport intersection results. Changes with pan/zoom.
```

```typescript
function useVisibleStaticLayers(bbox: BBox | null, visibleLayerIds: string[]) {
  const engine = useSpatialEngine();

  // Classify layers by WASM readiness
  const canonicalIds = visibleLayerIds.map(canonicalLayerId);
  const wasmReady = canonicalIds.filter(id => engine.queryReady([id]));
  const needsFallback = canonicalIds.filter(id => !engine.queryReady([id]));

  // Layer 2: WASM batch query (bbox-dependent)
  const wasmResult = useQuery({
    queryKey: ["static-layers-viewport", bbox, wasmReady.sort().join(",")],
    queryFn: () => engine.query(bbox!, wasmReady),
    enabled: bbox !== null && wasmReady.length > 0,
    staleTime: 5_000,
  });

  // Layer 1: FGB fallback per layer (bbox-independent, infinite cache)
  const fallbackResults = needsFallback.map(id =>
    useQuery({
      queryKey: ["fgb-asset", prefCodeForLayer(id), id],
      queryFn: () => fetchFgbLayer(id),
      enabled: bbox !== null,
      staleTime: Infinity,
      gcTime: Infinity,
    })
  );

  // Merge WASM + fallback results into Map<string, FeatureCollection>
  // ...
}
```

This preserves the current "fetch once, cache forever" FGB behavior while adding the batched WASM path.

---

## Phase 3: Value Extension

### 3.1 Two-Stage Layer Loading

**Boot (immediate)**:
- `admin-boundary` (always needed for boundaries)
- Static layers in the active theme (from URL `theme=safety` → safety layers)

**On-demand (lazy)**:
- When user toggles a new static layer ON, check if loaded in WASM
- If not loaded, send `load-layer` message to worker
- Worker fetches FGB, builds R-tree, responds `layer-loaded`
- Adapter updates `loadedLayers`
- Until loaded, layer uses FGB fallback path (Layer 1 cache)

**New worker messages:**
```typescript
// Main → Worker
{ type: "load-layer", id: string, url: string }

// Worker → Main (success)
{ type: "layer-loaded", id: string, count: number }

// Worker → Main (failure — includes layer id for isolation)
{ type: "layer-load-failed", id: string, error: string }
```

**Init changes:**
```typescript
// Before: load ALL layers at init
worker.postMessage({ type: "init", layers: ALL_11_LAYERS });

// After: load boot-critical only
const bootLayers = getBootLayers(activeTheme);
worker.postMessage({ type: "init", layers: bootLayers });
```

### 3.2 SpatialEngineProvider + Boot Theme Resolution

**Problem identified by Codex review:** Provider init happens before `useMapUrlState()` restores theme from URL. If provider reads theme at mount, deep-link themes are missed.

**Solution: Provider does NOT determine boot layers. It initializes the engine and waits for theme resolution.**

```typescript
// providers.tsx
export function SpatialEngineProvider({ children }: { children: ReactNode }) {
  // Phase 1: Just init the engine (no layers yet)
  useEffect(() => {
    spatialEngine.initEngine(); // Only loads WASM module, no layers
    return () => { /* session singleton, no dispose */ };
  }, []);

  return (
    <SpatialEngineContext.Provider value={spatialEngine}>
      {children}
    </SpatialEngineContext.Provider>
  );
}
```

**Boot layer loading happens in `useMapPage` AFTER URL state is resolved:**

```typescript
// use-map-page.ts
export function useMapPage() {
  useMapUrlState(); // Restores theme from URL first

  const engine = useSpatialEngine();
  const activeThemes = useUIStore((s) => s.activeThemes);

  // Boot layers: determined AFTER URL theme is resolved
  useEffect(() => {
    const bootLayers = getBootLayers(activeThemes);
    engine.loadLayers(bootLayers); // Sends load-layer messages
  }, []); // Only on mount, after URL restoration
  // ...
}
```

**Sequence:**
1. `SpatialEngineProvider` mounts → WASM module loaded (no data)
2. `useMapPage` mounts → `useMapUrlState()` runs → theme restored from URL
3. `useEffect` in `useMapPage` reads `activeThemes` → determines boot layers → sends `load-layer` messages
4. As layers load, `loadedLayers` updates → `useVisibleStaticLayers` switches from FGB fallback to WASM

This ensures deep-link themes are respected for boot optimization.

### 3.3 Shared Domain Crate

**Location**: `services/shared-domain/`

**Scope: Constants and pure computation ONLY.** No DTO shapes, no response structures. Frontend Zod schemas remain the API contract source of truth (per CLAUDE.md).

```
services/shared-domain/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── constants.rs     # RISK_WEIGHT_FLOOD=0.6, RISK_WEIGHT_STEEP=0.4, etc.
    ├── land_price.rs    # compute_land_price_stats(prices: &[i64]) → LandPriceStats
    ├── risk.rs          # compute_area_ratio(bbox, geometries) → f64
    │                    # compute_composite_risk(flood, steep) → f64
    ├── facilities.rs    # count_in_bbox(points, bbox) → u32
    └── zoning.rs        # compute_zoning_distribution(bbox, polygons) → HashMap
```

**Cargo.toml (fixed — no duplicate `[features]`):**
```toml
[package]
name = "realestate-shared-domain"
version = "0.1.0"
edition = "2021"

[dependencies]
geo = { version = "0.28", default-features = false }
serde = { version = "1.0", features = ["derive"], optional = true }

[features]
default = []
std = []
serde = ["dep:serde"]
```

**Migration path:**
1. Extract `services/wasm/src/stats.rs` pure functions into `shared-domain`
2. Extract constants from `services/wasm/src/lib.rs` (RISK_WEIGHT_*)
3. Extract matching logic from `services/backend/src/domain/`
4. Both `services/wasm/Cargo.toml` and `services/backend/Cargo.toml` add `realestate-shared-domain = { path = "../shared-domain" }`

### 3.4 Stats Data Ingestion (Future)

To achieve full WASM stats parity, API-layer data must be ingested into the WASM R-tree:

```typescript
// New worker message
{ type: "ingest-features", layerId: string, geojson: string }
```

When `useAreaData` returns fresh data for `landprice`/`schools`/`medical`/`zoning`/`steep-slope`, the batched hook forwards the FeatureCollection to the worker for R-tree ingestion. This makes `statsReady` achievable.

**This is a separate design track** — not included in Phase 3 implementation plan. It requires its own spec covering:
- Cache invalidation (viewport change → stale ingested data)
- Memory budget (API data duplicated in WASM heap)
- Incremental vs full re-ingestion

### 3.5 WASM Responsibility Matrix

| Responsibility | Owner | Notes |
|---|---|---|
| R-tree bbox intersection | WASM | Core competency, O(log n) |
| Static layer viewport query | WASM (batch) | Single roundtrip via `useVisibleStaticLayers` |
| Polygon area ratio | WASM via shared-domain | `risk::compute_area_ratio` |
| Facility counting | WASM | R-tree count query |
| Zoning distribution | WASM via shared-domain | `zoning::compute_zoning_distribution` |
| Land price stats | **Backend API (canonical)** | WASM preview deferred until data ingestion spec |
| Trend (time series) | Backend API | DB-dependent |
| TLS Score | Backend API | May use external data |
| Health check | Backend API | Server state |

---

## File Changes Summary

### Task 0 (Prerequisite)
| File | Change |
|------|--------|
| `services/frontend/src/lib/layer-ids.ts` | Create: canonical ID mapping + `canonicalLayerId()` |
| `services/frontend/src/lib/wasm/spatial-engine.ts` | Use `canonicalLayerId()` in init, query, loadedLayers |
| `services/frontend/src/hooks/use-static-layer.ts` | Use `canonicalLayerId()` for FGB paths |

### Phase 1 (Correctness)
| File | Change |
|------|--------|
| `services/frontend/src/lib/wasm/spatial-engine.ts` | Replace `_ready` with `loadedLayers`, add `queryReady`/`statsReady`, add Performance API marks, add `query-error`/`stats-error` handling |
| `services/frontend/src/lib/wasm/worker.ts` | Add Performance API marks, send `query-error`/`stats-error` with request `id` |
| `services/frontend/src/hooks/use-spatial-engine.ts` | Return `{ queryReady, statsReady, loadedLayers }` |
| `services/frontend/src/__tests__/spatial-engine.test.ts` | Create: test ready state separation, error isolation |

### Phase 2 (Boundary)
| File | Change |
|------|--------|
| `services/wasm/src/spatial_index.rs` | Add `get_features_as_value()` returning `serde_json::Value` |
| `services/wasm/src/lib.rs` | Change `query_layers_inner` to use `Value` instead of nested `String` |
| `services/frontend/src/lib/wasm/spatial-engine.ts` | Simplify `query-result` handler (single parse) |
| `services/frontend/src/hooks/use-visible-static-layers.ts` | Create: batched query hook with 2-layer cache |
| `services/frontend/src/hooks/use-static-layer.ts` | Deprecate |
| `services/frontend/src/components/map/layers/*.tsx` | All static layers: self-fetch → prop-receive |
| `services/frontend/src/components/map/layer-renderer.tsx` | Call `useVisibleStaticLayers`, pass data to children |

### Phase 3 (Value)
| File | Change |
|------|--------|
| `services/shared-domain/` | Create: shared Rust domain crate (constants + pure computation) |
| `services/wasm/Cargo.toml` | Add `shared-domain` dependency |
| `services/wasm/src/lib.rs` | Import stats from `shared-domain` |
| `services/wasm/src/stats.rs` | Remove (migrated to shared-domain) |
| `services/backend/Cargo.toml` | Add `shared-domain` dependency |
| `services/backend/src/domain/` | Import constants/formulas from `shared-domain` |
| `services/frontend/src/lib/wasm/spatial-engine.ts` | Add `loadLayers()`, `initEngine()` (split from `init()`) |
| `services/frontend/src/lib/wasm/worker.ts` | Add `load-layer` / `layer-loaded` / `layer-load-failed` messages |
| `services/frontend/src/components/providers.tsx` | Add `SpatialEngineProvider` (engine init only, no data) |
| `services/frontend/src/hooks/use-map-page.ts` | Move `spatialEngine.init()` → `engine.loadLayers(bootLayers)` after URL state resolution |

---

## Success Criteria

| Metric | Target | Measurement |
|--------|--------|-------------|
| wasm-init p95 | < 1s | `performance.measure('wasm-init')` |
| wasm-query p95 (5 layers) | < 16ms | `performance.measure('wasm-query-*')` |
| Worker roundtrips per viewport | 1 | DevTools Network/Performance |
| JSON parse count per query | 1 | Code inspection (no inner parse) |
| Boot layer count | 2-5 (theme-dependent) | `loadedLayers.size` at init-done |
| Fallback rate | 0% after full load | `log.info({ fallback_rate })` |
| Error isolation | Per-request | No cross-request rejection |
| FGB refetch on pan | 0 (cached) | TanStack Query devtools |

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| shared-domain `no_std` constraint limits geo crate usage | Can't use all geo features | geo supports `no_std` with feature flags |
| Lazy loading causes visible layer pop-in | UX degradation | FGB fallback renders immediately; WASM takes over seamlessly |
| JSON flattening increases peak memory in WASM | OOM on large datasets | Benchmark with full Tokyo dataset before/after |
| Provider singleton leaks memory on long sessions | Memory growth | Cap R-tree entries, monitor via Performance API |
| Backend/WASM shared-domain version drift | Incorrect stats | Workspace-level version, CI tests both targets |
| Layer ID mismatch causes silent query failures | Empty results | Task 0 normalization + test that all LAYERS ids resolve to valid canonical IDs |
| FGB fallback cache mixed with bbox query cache | Unnecessary refetches on pan | Two-layer cache architecture (Layer 1: asset, Layer 2: viewport) |
| Deep-link theme missed at boot | Suboptimal boot layer set | Provider does NOT pick boot layers; `useMapPage` picks after URL state resolution |
| Error blast radius on batch query failure | All visible layers fail | Request-scoped `query-error` with `id`; each pending resolved independently |

---

## Implementation Order

0. Task 0: Canonical layer ID normalization
1. Phase 1.1: Ready state separation + loadedLayers
2. Phase 1.2: Request-scoped error handling
3. Phase 1.4: Observability (Performance API)
4. Phase 1.3: Stats gating (document current limitation)
5. Phase 2.2: JSON flattening (Rust side)
6. Phase 2.1: `useVisibleStaticLayers` batch hook (2-layer cache)
7. Phase 2.3: Static layer component migration
8. Phase 3.3: Shared domain crate
9. Phase 3.1: Two-stage layer loading + `load-layer` messages
10. Phase 3.2: SpatialEngineProvider + boot sequence
