# Mapbox GL JS Migration Guide

Status: トリガー待ち (Phase 1 stays on MapLibre GL JS)
Last updated: 2026-03-23

## Background

The project currently uses MapLibre GL JS (`maplibre-gl@5.20.2`) via `react-map-gl@8.1.0`.
`react-map-gl` is a dual-renderer library — the same React components work with both
MapLibre and Mapbox GL JS by changing the import path from `react-map-gl/maplibre` to
`react-map-gl` (which defaults to Mapbox). This makes the migration a shallow find-and-replace
across a small number of files.

---

## 1. Trigger Conditions

Switch when **at least one** of the following is needed:

| Trigger | Why Mapbox is required |
|---|---|
| **Globe projection** | `projection: 'globe'` and atmospheric scattering are Mapbox GL JS v3 exclusives. MapLibre has no equivalent. |
| **Terrain 3D (Mapbox DEM tiles)** | The current terrain source uses the open `terrarium` encoding from `elevation-tiles-prod`. Mapbox's own `mapbox-terrain-dem-v1` provides higher resolution (512px) tiles and tighter Studio integration; required for Mapbox terrain styles. |
| **Mapbox Studio styles** | Studio-authored styles use Mapbox-hosted sources (`mapbox://`) which authenticate against Mapbox tokens and do not resolve in MapLibre. |
| **Standard/Satellite-Streets v12 styles** | Mapbox's next-gen Standard style, Satellite Streets v12, and Light/Dark v11 require Mapbox GL JS v3. |
| **Fog / Sky / Atmosphere effects** | Mapbox GL JS v3 `setFog()` and atmosphere layer are not in MapLibre. |

Do NOT migrate just for performance or style parity — MapLibre v5 is performant and the
current dark CARTO style looks correct as-is.

---

## 2. Migration Scope — All Files That Must Change

### 2a. Files with direct MapLibre package references (3 files)

These are the only files that reference MapLibre-specific import paths or the CSS bundle:

| File | Line(s) | Current import | Change to |
|---|---|---|---|
| `src/components/map/map-view.tsx` | 3 | `import "maplibre-gl/dist/maplibre-gl.css"` | `import "mapbox-gl/dist/mapbox-gl.css"` |
| `src/components/map/map-view.tsx` | 13-17 | `from "react-map-gl/maplibre"` (types + components) | `from "react-map-gl"` |
| `src/components/map/map-view.tsx` | 37 | `maplibregl.Map` (ref type) | `mapboxgl.Map` |
| `src/app/page.tsx` | 5 | `from "react-map-gl/maplibre"` (type only) | `from "react-map-gl"` |

### 2b. Layer components (24 files — same mechanical change each)

Every layer component in `src/components/map/layers/` imports:

```typescript
import { Layer, Source } from "react-map-gl/maplibre";
```

This must become:

```typescript
import { Layer, Source } from "react-map-gl";
```

Full list:

- `admin-boundary-layer.tsx`
- `did-layer.tsx`
- `fault-layer.tsx`
- `flood-history-layer.tsx`
- `flood-layer.tsx`
- `geology-layer.tsx`
- `landform-layer.tsx`
- `landprice-layer.tsx`
- `landslide-layer.tsx`
- `liquefaction-layer.tsx`
- `medical-layer.tsx`
- `park-layer.tsx`
- `population-mesh-layer.tsx`
- `railway-layer.tsx`
- `school-district-layer.tsx`
- `school-layer.tsx`
- `seismic-layer.tsx`
- `soil-layer.tsx`
- `station-layer.tsx`
- `steep-slope-layer.tsx`
- `tsunami-layer.tsx`
- `urban-plan-layer.tsx`
- `volcano-layer.tsx`
- `zoning-layer.tsx`

This change is purely mechanical — `Source` and `Layer` have identical APIs in both
renderers as provided by `react-map-gl`.

### 2c. Files with NO changes required

| File | Reason |
|---|---|
| `src/lib/layers.ts` | Pure data config — no map imports |
| `src/lib/constants.ts` | Style URL change only (see section 4) |
| `src/stores/map-store.ts` | No map library dependency |
| `src/hooks/use-map-url-state.ts` | No map library dependency |
| `src/components/map/popup-card.tsx` | No map library dependency |
| `src/components/map/year-slider.tsx` | No map library dependency |
| All feature API hooks | No map library dependency |

---

## 3. Package Changes

### Remove

```bash
pnpm remove maplibre-gl
```

### Add

```bash
pnpm add mapbox-gl
```

Note: `react-map-gl@8.x` already lists `mapbox-gl` as an optional peer dependency
alongside `maplibre-gl`. No version bump to `react-map-gl` is needed.

**Versions as of this writing:**

| Package | Current (MapLibre) | Target (Mapbox) |
|---|---|---|
| `maplibre-gl` | `^5.20.2` | removed |
| `mapbox-gl` | — | `^3.x` (latest stable) |
| `react-map-gl` | `^8.1.0` | `^8.1.0` (unchanged) |

---

## 4. API Differences — Breaking and Non-Breaking

### 4a. No breaking changes in the layer/source API

The `Source`, `Layer`, `Map`, `NavigationControl`, `MapLayerMouseEvent`,
`ViewStateChangeEvent`, and `MapEvent` APIs exposed through `react-map-gl` are stable
across both renderers. The component props used in this codebase (`mapStyle`, `longitude`,
`latitude`, `zoom`, `pitch`, `bearing`, `onMove`, `onMoveEnd`, `onClick`, `onLoad`,
`interactiveLayerIds`, `attributionControl`) are identical.

### 4b. `mapRef` type annotation

`map-view.tsx` line 37 uses the global namespace type directly:

```typescript
const mapRef = useRef<maplibregl.Map | null>(null);
```

Change to:

```typescript
import type mapboxgl from "mapbox-gl";
const mapRef = useRef<mapboxgl.Map | null>(null);
```

Or use the `react-map-gl` re-exported type to stay renderer-agnostic:

```typescript
import type { MapRef } from "react-map-gl";
const mapRef = useRef<MapRef | null>(null);
```

The `MapRef` approach is preferable because it does not bind to the underlying library type.

### 4c. Terrain source encoding

Current terrain setup in `map-view.tsx` `handleLoad`:

```typescript
map.addSource("terrain-dem", {
  type: "raster-dem",
  tiles: ["https://s3.amazonaws.com/elevation-tiles-prod/terrainrgb/{z}/{x}/{y}.png"],
  tileSize: 256,
  maxzoom: 15,
  encoding: "terrarium",
});
map.setTerrain({ source: "terrain-dem", exaggeration: 1.5 });
```

This open tile source continues to work with Mapbox GL JS. No change required unless
you want to switch to Mapbox-hosted terrain tiles (`mapbox://mapbox.mapbox-terrain-dem-v1`)
for higher resolution — that requires a Mapbox access token and `encoding: "mapbox"`.

### 4d. 3D buildings source

The current 3D buildings layer uses the CARTO `building` source layer, which is a
vector tile source already loaded via the CARTO style. This continues to work in
Mapbox GL JS. No change required. If switching to a Mapbox style, buildings are
available in Mapbox's own `composite` source under the `building` source layer.

### 4e. `setTerrain` availability

`map.setTerrain()` exists in both MapLibre GL JS v3+ and Mapbox GL JS v3+. No
compatibility issue.

### 4f. WebGL context recovery

The `webglcontextlost` / `webglcontextrestored` event listeners in `handleLoad` use
standard DOM canvas events and work identically on both renderers.

---

## 5. Style URL Changes

### Current style (`src/lib/constants.ts`)

```typescript
style: "https://basemaps.cartocdn.com/gl/dark-matter-gl-style/style.json",
```

CARTO's dark-matter style is a public JSON URL that works with both MapLibre and Mapbox GL
JS as-is. No change is strictly required.

However, if you want to use a Mapbox-hosted style after migrating:

| Intent | Mapbox Style URL |
|---|---|
| Dark (closest to current) | `mapbox://styles/mapbox/dark-v11` |
| Satellite + streets | `mapbox://styles/mapbox/satellite-streets-v12` |
| Next-gen Standard | `mapbox://styles/mapbox/standard` |
| Custom Studio style | `mapbox://styles/{username}/{style_id}` |

Mapbox style URLs require a valid `accessToken` prop on the `<Map>` component:

```typescript
<MapGL
  ...
  mapboxAccessToken={process.env.NEXT_PUBLIC_MAPBOX_TOKEN}
>
```

Store the token in `.env.local`:

```
NEXT_PUBLIC_MAPBOX_TOKEN=pk.eyJ1...
```

The `NEXT_PUBLIC_` prefix is intentional — Mapbox tokens are public-facing by design
(they are restricted by URL allowlist in the Mapbox dashboard).

---

## 6. Cost Estimate

Mapbox GL JS itself is free (MIT-licensed). Costs arise from Mapbox-hosted tile services.

### What generates Mapbox billable events

- **Map loads**: Each distinct page load that initializes a Mapbox GL JS map instance
- **Tile requests**: Only when using Mapbox-hosted styles (`mapbox://`) or Mapbox terrain tiles

### What does NOT generate billable events

- Using third-party tile sources (CARTO, OpenStreetMap, custom PostGIS)
- Using a local or self-hosted style JSON
- The `mapbox-gl` npm package itself

### Pricing tiers (as of 2026-03)

Pricing is denominated in "Map Loads":

| Tier | Monthly map loads | Cost |
|---|---|---|
| Free | 0 – 50,000 | $0 |
| Pay-as-you-go | 50,001 – 100,000 | $0.50 / 1,000 loads |
| Pay-as-you-go | 100,001+ | Reduces further with volume |

Reference: https://www.mapbox.com/pricing

### Estimate for this project

At Tokyo real estate SaaS launch scale (internal / small subscriber base):

| Scenario | Monthly active users | Est. map loads/user/day | Monthly loads | Monthly cost |
|---|---|---|---|---|
| Internal / beta | 20 | 5 | 3,000 | $0 (free tier) |
| Early SaaS (100 users) | 100 | 10 | 30,000 | $0 (free tier) |
| Growth (500 users) | 500 | 10 | 150,000 | ~$50 |

The free tier is 50,000 map loads/month. Costs become meaningful only at scale,
consistent with the TODOS.md note that "従量課金はSaaS収益でカバー" (usage billing
covered by SaaS revenue).

---

## 7. Estimated Effort

| Step | Files | Effort |
|---|---|---|
| `package.json`: swap packages | 1 | 5 min |
| `map-view.tsx`: CSS import + type imports + ref type | 1 | 10 min |
| `page.tsx`: update import path | 1 | 2 min |
| 24 layer components: import path find-and-replace | 24 | 5 min (single sed or IDE rename) |
| `constants.ts`: add `mapboxAccessToken` export (optional) | 1 | 5 min |
| Smoke test: map loads, layers render, click-inspect works | — | 30 min |

**Total: ~1 hour** (the "S effort" estimate in TODOS.md is accurate)

The find-and-replace from `react-map-gl/maplibre` to `react-map-gl` across the 24 layer
files can be done in one command:

```bash
find services/frontend/src/components/map/layers -name "*.tsx" \
  -exec sed -i '' 's|react-map-gl/maplibre|react-map-gl|g' {} \;
```

---

## 8. Risk Assessment

**Overall risk: Low**

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Layer paint expressions stop working | Very low | High | MapGL expressions spec is identical between MapLibre v5 and Mapbox GL JS v3 |
| CARTO style stops rendering | Very low | Medium | CARTO style JSON is renderer-agnostic; only hosted sources differ |
| `maplibregl.Map` type errors after swap | Certain | Low | One-line fix: change ref type to `mapboxgl.Map` or `MapRef` |
| Terrain tiles stop rendering | Very low | Low | Open terrarium tiles work with both renderers |
| Billable surprise | Low | Low | Free tier covers all pre-growth scenarios |
| Breaking change in `react-map-gl` | Very low | Low | Library is stable; both renderer paths are first-class |

The only guaranteed code change beyond the import swaps is the `mapRef` type annotation
in `map-view.tsx` (one line, compile error surfaces it immediately).

---

## 9. Step-by-Step Execution Checklist

When the trigger condition is met:

- [ ] Obtain Mapbox access token from https://account.mapbox.com and add to `.env.local` as `NEXT_PUBLIC_MAPBOX_TOKEN`
- [ ] `pnpm remove maplibre-gl && pnpm add mapbox-gl`
- [ ] In `map-view.tsx`: replace `maplibre-gl/dist/maplibre-gl.css` with `mapbox-gl/dist/mapbox-gl.css`
- [ ] In `map-view.tsx`: replace all `react-map-gl/maplibre` imports with `react-map-gl`
- [ ] In `map-view.tsx`: fix `mapRef` type (`maplibregl.Map` → `mapboxgl.Map` or `MapRef`)
- [ ] In `map-view.tsx`: add `mapboxAccessToken={process.env.NEXT_PUBLIC_MAPBOX_TOKEN}` to `<MapGL>` props if using Mapbox-hosted styles
- [ ] In `page.tsx`: replace `react-map-gl/maplibre` import with `react-map-gl`
- [ ] In all 24 layer files: replace `react-map-gl/maplibre` with `react-map-gl` (use the `sed` one-liner above)
- [ ] (Optional) In `constants.ts`: update `MAP_CONFIG.style` to a Mapbox style URL
- [ ] Run `pnpm tsc --noEmit` — expect zero errors
- [ ] Run `pnpm biome check .` — expect zero errors
- [ ] Run `pnpm vitest run` — all tests pass
- [ ] Manual smoke test: map loads, all layer toggles work, click-inspect popups appear, terrain renders, 3D buildings render
- [ ] Verify Mapbox dashboard token usage is not exceeding free tier
