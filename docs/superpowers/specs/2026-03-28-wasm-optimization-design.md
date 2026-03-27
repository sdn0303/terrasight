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

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│ Frontend (React 19)                                      │
│                                                          │
│  ┌──────────────────┐    ┌───────────────────────────┐  │
│  │ SpatialEngine    │    │ useVisibleStaticLayers()   │  │
│  │ Provider         │◄───│ single hook, batched query │  │
│  │ (app-level       │    └───────────────────────────┘  │
│  │  singleton)      │                                    │
│  └────────┬─────────┘    ┌───────────────────────────┐  │
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
│  │   (stats, consts)│                                   │
│  └──────────────────┘                                    │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ Backend (Rust Axum)                                      │
│                                                          │
│  handler → usecase → domain ← infra                     │
│                        │                                 │
│                   shared-domain                          │
│                   (same crate as WASM)                   │
└─────────────────────────────────────────────────────────┘
```

---

## Phase 1: Correctness

### 1.1 Ready State Separation

**Current**: `_ready: boolean` — set to `true` on any `init-done`, even with partial loads.

**New**: Replace with `loadedLayers: Set<string>` derived from `init-done` counts.

```typescript
// spatial-engine.ts
class SpatialEngineAdapter {
  private loadedLayers = new Set<string>();

  queryReady(layerIds: string[]): boolean {
    return layerIds.every(id => this.loadedLayers.has(id));
  }

  get statsReady(): boolean {
    return STATS_REQUIRED_LAYERS.every(id => this.loadedLayers.has(id));
  }
}

const STATS_REQUIRED_LAYERS = [
  "landprice", "flood-history", "steep-slope",
  "schools", "medical", "zoning",
];
```

**Consumer hook change**: `useSpatialEngineReady()` → `useSpatialEngine()` returning `{ queryReady, statsReady, loadedLayers }`.

### 1.2 Stats Gating

- WASM stats path stays disabled (current state from X-02 fix)
- When `statsReady === true`, WASM stats become available as "preview"
- Canonical source remains `/api/stats` until parity test passes
- Parity test: vitest that calls both paths with same bbox, asserts delta < 0.01

### 1.3 Observability (Performance API)

```typescript
// Wrap in spatial-engine.ts
performance.mark("wasm-init-start");
// ... init ...
performance.mark("wasm-init-done");
performance.measure("wasm-init", "wasm-init-start", "wasm-init-done");

// Per-layer in worker.ts
performance.mark(`layer-load-${id}-start`);
// ... load_layer ...
performance.mark(`layer-load-${id}-done`);
performance.measure(`layer-load-${id}`, ...);

// Per-query
performance.mark(`wasm-query-${queryId}-start`);
// ... query_layers ...
performance.mark(`wasm-query-${queryId}-done`);
performance.measure(`wasm-query-${queryId}`, ...);
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
... (11 layers)
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

### 2.3 Fallback Path Preservation

When WASM is not ready (`queryReady === false` for a layer), fallback to direct FGB fetch (current `flatgeobuf/deserialize` path). The batched hook handles this:

```typescript
// In useVisibleStaticLayers
const wasmLayers = ids.filter(id => engine.queryReady([id]));
const fallbackLayers = ids.filter(id => !engine.queryReady([id]));

// WASM batch for ready layers
const wasmResult = await engine.query(bbox, wasmLayers);

// FGB fallback for not-yet-loaded layers
const fallbackResults = await Promise.all(
  fallbackLayers.map(id => fetchFgbLayer(id))
);
```

---

## Phase 3: Value Extension

### 3.1 Two-Stage Layer Loading

**Boot (immediate)**:
- `admin-boundary` (always needed for boundaries)
- Layers in the active theme (from URL `theme=safety` → safety layers)

**On-demand (lazy)**:
- When user toggles a new layer ON, check if loaded
- If not loaded, send `load-layer` message to worker
- Worker fetches FGB, builds R-tree, responds `layer-loaded`
- Adapter updates `loadedLayers`

**New worker messages:**
```typescript
// Main → Worker
{ type: "load-layer", id: string, url: string }

// Worker → Main
{ type: "layer-loaded", id: string, count: number }
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

### 3.2 SpatialEngineProvider

Move singleton lifecycle from `useMapPage` to app-level Provider:

```typescript
// providers.tsx
export function SpatialEngineProvider({ children }: { children: ReactNode }) {
  useEffect(() => {
    spatialEngine.init();
    // No dispose on unmount — session singleton
    return () => {
      // Only on page unload
    };
  }, []);

  return <SpatialEngineContext.Provider value={spatialEngine}>
    {children}
  </SpatialEngineContext.Provider>;
}

// Hook
export function useSpatialEngine(): SpatialEngineAdapter {
  return useContext(SpatialEngineContext);
}
```

Remove `spatialEngine.init()` and `spatialEngine.dispose()` from `useMapPage`.

### 3.3 Shared Domain Crate

**Location**: `services/shared-domain/`

```
services/shared-domain/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── constants.rs     # RISK_WEIGHT_FLOOD=0.6, RISK_WEIGHT_STEEP=0.4, etc.
    ├── land_price.rs    # compute_land_price_stats(prices) → LandPriceStats
    ├── risk.rs          # compute_area_ratio(bbox, geometries) → f64
    │                    # compute_composite_risk(flood, steep) → f64
    ├── facilities.rs    # count_facilities(bbox, points) → u32
    ├── zoning.rs        # compute_zoning_distribution(bbox, polygons) → HashMap
    └── types.rs         # LandPriceStats, RiskStats, FacilityStats, ZoningDist
```

**Cargo.toml:**
```toml
[package]
name = "realestate-shared-domain"
version = "0.1.0"
edition = "2021"

[features]
default = []
std = []  # Backend enables this

[dependencies]
geo = { version = "0.28", default-features = false }
serde = { version = "1.0", features = ["derive"], optional = true }

[features]
serde = ["dep:serde"]
```

**Migration path:**
1. Extract `services/wasm/src/stats.rs` functions into `shared-domain`
2. Extract constants from `services/wasm/src/lib.rs` (RISK_WEIGHT_*)
3. Extract matching logic from `services/backend/src/domain/`
4. Both `services/wasm/Cargo.toml` and `services/backend/Cargo.toml` add `realestate-shared-domain = { path = "../shared-domain" }`

### 3.4 WASM Responsibility Matrix

| Responsibility | Owner | Notes |
|---|---|---|
| R-tree bbox intersection | WASM | Core competency, O(log n) |
| Static layer viewport query | WASM (batch) | Single roundtrip via `useVisibleStaticLayers` |
| Polygon area ratio | WASM via shared-domain | `risk::compute_area_ratio` |
| Facility counting | WASM | R-tree count query |
| Zoning distribution | WASM via shared-domain | `zoning::compute_zoning_distribution` |
| Land price stats | WASM (preview) / API (canonical) | Shared formula, parity gated |
| Trend (time series) | Backend API | DB-dependent |
| TLS Score | Backend API | May use external data |
| Health check | Backend API | Server state |

---

## File Changes Summary

### Phase 1 (Correctness)
| File | Change |
|------|--------|
| `services/frontend/src/lib/wasm/spatial-engine.ts` | Replace `_ready` with `loadedLayers`, add `queryReady`/`statsReady`, add Performance API marks |
| `services/frontend/src/lib/wasm/worker.ts` | Add Performance API marks for layer load + query |
| `services/frontend/src/hooks/use-spatial-engine.ts` | Return `{ queryReady, statsReady, loadedLayers }` |
| `services/frontend/src/__tests__/spatial-engine.test.ts` | Create: test ready state separation, stats gating |

### Phase 2 (Boundary)
| File | Change |
|------|--------|
| `services/wasm/src/spatial_index.rs` | Add `get_features_as_value()` returning `serde_json::Value` |
| `services/wasm/src/lib.rs` | Change `query_layers_inner` to use `Value` instead of nested `String` |
| `services/frontend/src/lib/wasm/spatial-engine.ts` | Simplify `query-result` handler (single parse) |
| `services/frontend/src/hooks/use-visible-static-layers.ts` | Create: batched query hook |
| `services/frontend/src/hooks/use-static-layer.ts` | Deprecate |
| `services/frontend/src/components/map/layers/*.tsx` | All static layers: self-fetch → prop-receive |
| `services/frontend/src/components/map/layer-renderer.tsx` | Call `useVisibleStaticLayers`, pass data to children |

### Phase 3 (Value)
| File | Change |
|------|--------|
| `services/shared-domain/` | Create: shared Rust domain crate |
| `services/wasm/Cargo.toml` | Add `shared-domain` dependency |
| `services/wasm/src/lib.rs` | Import stats from `shared-domain` |
| `services/wasm/src/stats.rs` | Remove (migrated to shared-domain) |
| `services/backend/Cargo.toml` | Add `shared-domain` dependency |
| `services/backend/src/domain/` | Import constants/formulas from `shared-domain` |
| `services/frontend/src/lib/wasm/spatial-engine.ts` | Add `loadLayer()`, boot-critical init |
| `services/frontend/src/lib/wasm/worker.ts` | Add `load-layer` / `layer-loaded` messages |
| `services/frontend/src/components/providers.tsx` | Add `SpatialEngineProvider` |
| `services/frontend/src/hooks/use-map-page.ts` | Remove `spatialEngine.init()/dispose()` |

---

## Success Criteria

| Metric | Target | Measurement |
|--------|--------|-------------|
| wasm-init p95 | < 1s | `performance.measure('wasm-init')` |
| wasm-query p95 (5 layers) | < 16ms | `performance.measure('wasm-query-*')` |
| Worker roundtrips per viewport | 1 | DevTools Network/Performance |
| JSON parse count per query | 1 | Code inspection (no inner parse) |
| Stats parity delta | 0 (or < 0.01) | Vitest parity test |
| Boot layer count | 2-5 (theme-dependent) | `loadedLayers.size` at init-done |
| Fallback rate | 0% after full load | `log.info({ fallback_rate })` |

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| shared-domain `no_std` constraint limits geo crate usage | Can't use all geo features | geo supports `no_std` with feature flags |
| Lazy loading causes visible layer pop-in | UX degradation | Show skeleton/loading indicator per layer; boot theme layers eagerly |
| JSON flattening increases peak memory in WASM | OOM on large datasets | Benchmark with full Tokyo dataset before/after |
| Provider singleton leaks memory on long sessions | Memory growth | Cap R-tree entries, monitor via Performance API |
| Backend/WASM shared-domain version drift | Incorrect stats | Workspace-level version, CI tests both targets |

---

## Implementation Order

1. Phase 1.1: Ready state separation
2. Phase 1.3: Observability (Performance API)
3. Phase 1.2: Stats gating + parity test
4. Phase 2.2: JSON flattening (Rust side)
5. Phase 2.1: `useVisibleStaticLayers` batch hook
6. Phase 2.3: Static layer component migration
7. Phase 3.3: Shared domain crate
8. Phase 3.1: Two-stage layer loading
9. Phase 3.2: SpatialEngineProvider
10. Phase 3.4: Stats preview enablement
