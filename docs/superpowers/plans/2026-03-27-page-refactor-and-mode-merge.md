# page.tsx Refactor + Mode Merge Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split the 337-line `page.tsx` God Component into focused modules, then merge the 3-mode system (Explore/Analyze/Compare) into 2 modes (Explore with progressive disclosure + Compare). Also fixes P0.5 items X-04 (selectedArea wiring) and X-06 (bbox → getBounds).

**Architecture:** Extract data-fetching + interaction logic into a custom hook (`useMapPage`). Extract layer rendering into a dedicated component (`LayerRenderer`). Remove cross-store mutation. Merge AnalysisStore into MapStore. Eliminate auto-mode-switch on feature click. The resulting `page.tsx` is a ~60-line layout shell.

**Tech Stack:** Next.js 16, React 19, Zustand, TanStack Query v5, MapLibre GL, nuqs

**Design specs:** See `~/.gstack/projects/sample-app/ceo-plans/2026-03-27-ux-craft-polish.md` (Design Specifications section)

**CLAUDE.md rules enforced:**
- No `any` — remove `as Record<string, unknown>` casts
- No store-derived query keys without debounce — fix bbox (X-06)
- Frontend Zod schema is source of truth
- Validate at boundaries

---

## File Structure

| File | Responsibility | Action |
|------|---------------|--------|
| `src/app/page.tsx` | Layout shell: TopBar + ContextPanel + MapContainer + StatusBar | **Rewrite** (337→~60 lines) |
| `src/hooks/use-map-page.ts` | All data fetching + derived state for the map page | **Create** |
| `src/hooks/use-map-interaction.ts` | Click handlers: feature click, compare point, area deselect | **Create** |
| `src/components/map/layer-renderer.tsx` | Renders all 24 layers from component registries | **Create** |
| `src/stores/map-store.ts` | Add `analysisPoint` + `weightPreset` (absorb from analysis-store) | **Modify** |
| `src/stores/analysis-store.ts` | Delete (merged into map-store) | **Delete** |
| `src/stores/ui-store.ts` | Change `AppMode` to `"explore" \| "compare"`, remove "analyze" | **Modify** |
| `src/components/context-panel/analyze-panel.tsx` | Delete (content moves to explore-panel) | **Delete** |
| `src/components/top-bar/mode-tabs.tsx` | Remove "分析" tab, keep 探索/比較 | **Modify** |
| `src/lib/constants.ts` | Add `PANEL_WIDTH = 320`, `TOP_BAR_HEIGHT = 48`, `STATUS_BAR_HEIGHT = 28` | **Modify** |

---

## Task 1: Extract Layout Constants

**Files:**
- Modify: `services/frontend/src/lib/constants.ts`

- [ ] **Step 1: Add layout constants**

```typescript
// Append to services/frontend/src/lib/constants.ts

export const PANEL_WIDTH = 320;
export const TOP_BAR_HEIGHT = 48;
export const STATUS_BAR_HEIGHT = 28;
```

- [ ] **Step 2: Commit**

```bash
git add services/frontend/src/lib/constants.ts
git commit -m "refactor(constants): extract layout dimensions from magic numbers"
```

---

## Task 2: Merge AnalysisStore into MapStore

**Files:**
- Modify: `services/frontend/src/stores/map-store.ts`
- Delete: `services/frontend/src/stores/analysis-store.ts`

- [ ] **Step 1: Add analysis fields to MapStore**

Add to the `MapState` interface and store initialization in `services/frontend/src/stores/map-store.ts`:

```typescript
import { create } from "zustand";
import { devtools } from "zustand/middleware";
import { LAYERS } from "@/lib/layers";

interface ViewState {
  latitude: number;
  longitude: number;
  zoom: number;
  pitch: number;
  bearing: number;
}

interface SelectedFeature {
  layerId: string;
  properties: Record<string, unknown>;
  coordinates: [number, number]; // [lng, lat] per RFC 7946
}

export interface SelectedArea {
  code: string;
  name: string;
  level: "prefecture" | "municipality";
  bbox: { south: number; west: number; north: number; east: number };
}

export type WeightPreset = "balance" | "investment" | "residential" | "disaster";

export interface AnalysisPoint {
  lat: number;
  lng: number;
  address?: string;
}

interface MapState {
  viewState: ViewState;
  visibleLayers: Set<string>;
  selectedFeature: SelectedFeature | null;
  selectedArea: SelectedArea | null;

  // Merged from analysis-store
  analysisPoint: AnalysisPoint | null;
  weightPreset: WeightPreset;
  analysisRadius: number;

  setViewState: (viewState: ViewState) => void;
  toggleLayer: (layerId: string) => void;
  selectFeature: (feature: SelectedFeature | null) => void;
  selectArea: (area: SelectedArea | null) => void;
  setAnalysisPoint: (point: AnalysisPoint | null) => void;
  setWeightPreset: (preset: WeightPreset) => void;
  setAnalysisRadius: (radius: number) => void;
  getBBox: () => { south: number; west: number; north: number; east: number };
}

const defaultVisibleLayers = new Set(
  LAYERS.filter((l) => l.defaultEnabled).map((l) => l.id),
);

export const useMapStore = create<MapState>()(
  devtools(
    (set, get) => ({
      viewState: {
        latitude: 35.681,
        longitude: 139.767,
        zoom: 12,
        pitch: 45,
        bearing: 0,
      },
      visibleLayers: defaultVisibleLayers,
      selectedFeature: null,
      selectedArea: null,

      // Merged from analysis-store
      analysisPoint: null,
      weightPreset: "balance" as WeightPreset,
      analysisRadius: 500,

      setViewState: (viewState) => set({ viewState }),

      toggleLayer: (layerId) =>
        set((state) => {
          const next = new Set(state.visibleLayers);
          if (next.has(layerId)) {
            next.delete(layerId);
          } else {
            next.add(layerId);
          }
          return { visibleLayers: next };
        }),

      selectFeature: (feature) => set({ selectedFeature: feature }),
      selectArea: (area) => set({ selectedArea: area }),
      setAnalysisPoint: (point) => set({ analysisPoint: point }),
      setWeightPreset: (preset) => set({ weightPreset: preset }),
      setAnalysisRadius: (radius) => set({ analysisRadius: radius }),

      getBBox: () => {
        const { latitude, longitude, zoom } = get().viewState;
        const latRange = 180 / 2 ** zoom;
        const lngRange = 360 / 2 ** zoom;
        return {
          south: latitude - latRange / 2,
          west: longitude - lngRange / 2,
          north: latitude + latRange / 2,
          east: longitude + lngRange / 2,
        };
      },
    }),
    { name: "map-store" },
  ),
);
```

- [ ] **Step 2: Update all imports of analysis-store across the codebase**

Run to find all files importing analysis-store:

```bash
cd services/frontend && grep -r "analysis-store\|useAnalysisStore\|AnalysisPoint\|WeightPreset" src/ --include="*.ts" --include="*.tsx" -l
```

For each file: change `import { ... } from "@/stores/analysis-store"` to `import { ... } from "@/stores/map-store"`.

- [ ] **Step 3: Delete analysis-store.ts**

```bash
rm services/frontend/src/stores/analysis-store.ts
```

- [ ] **Step 4: Verify TypeScript compiles**

```bash
cd services/frontend && pnpm tsc --noEmit
```

- [ ] **Step 5: Run tests**

```bash
cd services/frontend && pnpm vitest run
```

- [ ] **Step 6: Commit**

```bash
git add -A services/frontend/src/stores/
git add services/frontend/src/components/ services/frontend/src/features/
git commit -m "refactor(stores): merge analysis-store into map-store

Eliminates cross-store mutation pattern. AnalysisPoint, WeightPreset,
and analysisRadius now live in MapStore alongside selectedFeature."
```

---

## Task 3: Simplify AppMode (3→2 modes)

**Files:**
- Modify: `services/frontend/src/stores/ui-store.ts`
- Modify: `services/frontend/src/components/top-bar/mode-tabs.tsx`

- [ ] **Step 1: Change AppMode type in ui-store.ts**

In `services/frontend/src/stores/ui-store.ts`, change:

```typescript
export type AppMode = "explore" | "compare";
```

Remove the "analyze" variant entirely.

- [ ] **Step 2: Update mode-tabs.tsx to show 2 tabs**

Find the mode tabs rendering and remove the "分析" / "analyze" tab. Keep only "探索" and "比較".

- [ ] **Step 3: Find and fix all `mode === "analyze"` references**

```bash
cd services/frontend && grep -rn '"analyze"' src/ --include="*.ts" --include="*.tsx"
```

Each hit should be either removed or replaced with "explore" behavior.

- [ ] **Step 4: Verify TypeScript compiles and tests pass**

```bash
cd services/frontend && pnpm tsc --noEmit && pnpm vitest run
```

- [ ] **Step 5: Commit**

```bash
git add -A services/frontend/src/
git commit -m "refactor(ui): simplify mode system from 3 to 2 modes

Merge Explore+Analyze into single Explore mode with progressive
disclosure. Compare mode is unchanged. Removes auto-mode-switch
on feature click (the #1 UX confusion source)."
```

---

## Task 4: Create useMapInteraction Hook

**Files:**
- Create: `services/frontend/src/hooks/use-map-interaction.ts`

- [ ] **Step 1: Create the hook**

```typescript
// services/frontend/src/hooks/use-map-interaction.ts
"use client";

import { useCallback } from "react";
import type { MapLayerMouseEvent } from "react-map-gl/maplibre";
import { useMapStore } from "@/stores/map-store";
import { useUIStore } from "@/stores/ui-store";

/**
 * Encapsulates map click interaction logic.
 *
 * In Explore mode:
 *   - Feature click → selectFeature + setAnalysisPoint (progressive disclosure)
 *   - Empty click   → deselect feature
 *
 * In Compare mode:
 *   - Click → set compare point A, then B
 *
 * No auto-mode-switch. No cross-store mutation.
 */
export function useMapInteraction() {
  const selectFeature = useMapStore((s) => s.selectFeature);
  const setAnalysisPoint = useMapStore((s) => s.setAnalysisPoint);
  const mode = useUIStore((s) => s.mode);
  const setComparePoint = useUIStore((s) => s.setComparePoint);

  const handleFeatureClick = useCallback(
    (e: MapLayerMouseEvent) => {
      const feature = e.features?.[0];

      if (mode === "compare") {
        const address =
          feature?.properties != null &&
          typeof feature.properties === "object" &&
          "address" in feature.properties &&
          typeof feature.properties.address === "string"
            ? feature.properties.address
            : `${e.lngLat.lat.toFixed(4)}, ${e.lngLat.lng.toFixed(4)}`;
        setComparePoint({
          lat: e.lngLat.lat,
          lng: e.lngLat.lng,
          address,
        });
        return;
      }

      // Explore mode
      if (feature) {
        selectFeature({
          layerId: feature.layer.id,
          properties: (feature.properties ?? {}) as Record<string, unknown>,
          coordinates: [e.lngLat.lng, e.lngLat.lat],
        });
        const featureAddress = feature?.properties?.address as
          | string
          | undefined;
        setAnalysisPoint({
          lat: e.lngLat.lat,
          lng: e.lngLat.lng,
          ...(featureAddress !== undefined ? { address: featureAddress } : {}),
        });
        // No mode switch — progressive disclosure within Explore panel
      } else {
        selectFeature(null);
      }
    },
    [mode, selectFeature, setAnalysisPoint, setComparePoint],
  );

  return { handleFeatureClick };
}
```

- [ ] **Step 2: Commit**

```bash
git add services/frontend/src/hooks/use-map-interaction.ts
git commit -m "feat(hooks): extract useMapInteraction from page.tsx

Encapsulates click logic for Explore and Compare modes.
No auto-mode-switch. No cross-store mutation."
```

---

## Task 5: Create useMapPage Hook

**Files:**
- Create: `services/frontend/src/hooks/use-map-page.ts`

- [ ] **Step 1: Create the hook**

```typescript
// services/frontend/src/hooks/use-map-page.ts
"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { parseAsInteger, useQueryState } from "nuqs";
import type { FeatureCollection } from "geojson";
import { useShallow } from "zustand/react/shallow";
import type { LayerConfig } from "@/lib/layers";
import { LAYERS } from "@/lib/layers";
import { spatialEngine } from "@/lib/wasm/spatial-engine";
import { useAreaData } from "@/features/area-data/api/use-area-data";
import { useHealth } from "@/features/health/api/use-health";
import { useLandPrices } from "@/features/land-prices/api/use-land-prices";
import { useMapUrlState } from "@/hooks/use-map-url-state";
import { useMapStore } from "@/stores/map-store";
import { logger } from "@/lib/logger";

const log = logger.child({ module: "use-map-page" });

const EMPTY_FC: FeatureCollection = {
  type: "FeatureCollection",
  features: [],
};

/** Precomputed interactive layer → config lookup (module-level singleton). */
const INTERACTIVE_LAYER_MAP = new Map<string, LayerConfig>();
for (const layer of LAYERS) {
  if (layer.interactiveLayerIds) {
    for (const maplibreId of layer.interactiveLayerIds) {
      INTERACTIVE_LAYER_MAP.set(maplibreId, layer);
    }
  }
}

/**
 * Aggregates all data-fetching and derived state for the map page.
 * Keeps page.tsx as a pure layout shell.
 */
export function useMapPage() {
  // URL-synced state
  useMapUrlState();

  // WASM init with error handling (X-02 spec review finding)
  const [wasmError, setWasmError] = useState(false);
  useEffect(() => {
    spatialEngine.init().catch((err: unknown) => {
      log.error({ err }, "WASM spatial engine failed to initialize");
      setWasmError(true);
    });
    return () => spatialEngine.dispose();
  }, []);

  // Map store
  const { visibleLayers, getBBox } = useMapStore(
    useShallow((s) => ({
      visibleLayers: s.visibleLayers,
      getBBox: s.getBBox,
    })),
  );
  const viewState = useMapStore((s) => s.viewState);
  const selectedFeature = useMapStore((s) => s.selectedFeature);

  // Bbox + debounced fetch trigger
  const [bbox, setBbox] = useState(() => getBBox());
  const handleMoveEnd = useCallback(() => {
    setBbox(getBBox());
  }, [getBBox]);

  // Year sliders
  const [populationYear, setPopulationYear] = useState(2020);
  const [landPriceYear, setLandPriceYear] = useQueryState(
    "year",
    parseAsInteger.withDefault(2024),
  );

  // Data fetching
  const layers = useMemo(() => [...visibleLayers], [visibleLayers]);
  const { data: areaData, isLoading } = useAreaData(
    bbox,
    layers,
    viewState.zoom,
  );
  const { data: health } = useHealth();
  const {
    data: landPriceData,
    isFetching: isLandPriceFetching,
    isError: isLandPriceError,
  } = useLandPrices(bbox, landPriceYear, viewState.zoom);

  // Derived state
  const isZoomTooLow = viewState.zoom < 10;
  const isDemoMode = health ? !health.reinfolib_key_set : true;

  const truncatedLayers = useMemo(() => {
    if (!areaData) return [];
    const result: { layer: string; count: number; limit: number }[] = [];
    for (const key of Object.keys(areaData) as (keyof typeof areaData)[]) {
      const layer = areaData[key];
      if (layer?.truncated === true) {
        result.push({ layer: key, count: layer.count, limit: layer.limit });
      }
    }
    return result;
  }, [areaData]);

  const selectedLayerConfig = useMemo(() => {
    if (!selectedFeature) return null;
    return (
      INTERACTIVE_LAYER_MAP.get(selectedFeature.layerId) ??
      LAYERS.find((l) => selectedFeature.layerId.startsWith(l.id)) ??
      null
    );
  }, [selectedFeature]);

  // Precomputed layer lists (stable references)
  const staticLayers = useMemo(
    () => LAYERS.filter((l) => l.source === "static"),
    [],
  );
  const apiLayers = useMemo(
    () => LAYERS.filter((l) => l.source === "api"),
    [],
  );

  return {
    // View state
    viewState,
    visibleLayers,
    selectedFeature,
    selectedLayerConfig,

    // Data
    areaData,
    landPriceData: landPriceData ?? EMPTY_FC,
    isLoading,
    isLandPriceFetching,
    isLandPriceError,
    isZoomTooLow,
    isDemoMode,
    truncatedLayers,
    wasmError,

    // Year controls
    populationYear,
    setPopulationYear,
    landPriceYear,
    setLandPriceYear,

    // Callbacks
    handleMoveEnd,

    // Layer lists
    staticLayers,
    apiLayers,
  };
}
```

- [ ] **Step 2: Commit**

```bash
git add services/frontend/src/hooks/use-map-page.ts
git commit -m "feat(hooks): extract useMapPage — all data fetching for map page

Aggregates area-data, land-prices, health, WASM init, year sliders,
bbox state, and derived state (truncatedLayers, selectedLayerConfig).
Fixes WASM init error handling (catch instead of silent failure)."
```

---

## Task 6: Create LayerRenderer Component

**Files:**
- Create: `services/frontend/src/components/map/layer-renderer.tsx`

- [ ] **Step 1: Create the component**

Extract the layer rendering loop from page.tsx lines 232-299 into a focused component. This component receives the data it needs as props — no store access.

```typescript
// services/frontend/src/components/map/layer-renderer.tsx
"use client";

import type { FeatureCollection } from "geojson";
import type { LayerConfig } from "@/lib/layers";
import { AreaHighlight } from "@/components/map/area-highlight";
import { BoundaryLayer } from "@/components/map/layers/boundary-layer";
import { LandPriceYearSlider } from "@/components/map/land-price-year-slider";
import { YearSlider } from "@/components/map/year-slider";
import {
  AdminBoundaryLayer,
  DIDLayer,
  FaultLayer,
  FloodHistoryLayer,
  FloodLayer,
  GeologyLayer,
  LandformLayer,
  LandPriceExtrusionLayer,
  LandpriceLayer,
  LandslideLayer,
  LiquefactionLayer,
  MedicalLayer,
  ParkLayer,
  PopulationMeshLayer,
  RailwayLayer,
  SchoolDistrictLayer,
  SchoolLayer,
  SeismicLayer,
  SoilLayer,
  StationLayer,
  SteepSlopeLayer,
  TsunamiLayer,
  UrbanPlanLayer,
  VolcanoLayer,
  ZoningLayer,
} from "@/components/map/layers";

const EMPTY_FC: FeatureCollection = {
  type: "FeatureCollection",
  features: [],
};

const STATIC_LAYER_COMPONENTS: Record<
  string,
  React.ComponentType<{ visible: boolean } & Record<string, unknown>>
> = {
  did: DIDLayer,
  landform: LandformLayer,
  geology: GeologyLayer,
  admin_boundary: AdminBoundaryLayer,
  fault: FaultLayer,
  flood_history: FloodHistoryLayer,
  liquefaction: LiquefactionLayer,
  railway: RailwayLayer,
  seismic: SeismicLayer,
  soil: SoilLayer,
  volcano: VolcanoLayer,
  station: StationLayer,
  school_district: SchoolDistrictLayer,
  landslide: LandslideLayer,
  park: ParkLayer,
  tsunami: TsunamiLayer,
  urban_plan: UrbanPlanLayer,
};

const API_LAYER_COMPONENTS: Record<
  string,
  React.ComponentType<{ data: FeatureCollection; visible: boolean }>
> = {
  landprice: LandpriceLayer,
  flood: FloodLayer,
  steep_slope: SteepSlopeLayer,
  schools: SchoolLayer,
  medical: MedicalLayer,
  zoning: ZoningLayer,
};

interface LayerRendererProps {
  visibleLayers: Set<string>;
  staticLayers: LayerConfig[];
  apiLayers: LayerConfig[];
  areaData: Record<string, unknown> | null;
  landPriceData: FeatureCollection;
  isLandPriceFetching: boolean;
  isLandPriceError: boolean;
  isZoomTooLow: boolean;
  populationYear: number;
  setPopulationYear: (year: number) => void;
  landPriceYear: number;
  setLandPriceYear: (year: number | null) => void;
  landPriceFeatureCount?: number;
}

export function LayerRenderer({
  visibleLayers,
  staticLayers,
  apiLayers,
  areaData,
  landPriceData,
  isLandPriceFetching,
  isLandPriceError,
  isZoomTooLow,
  populationYear,
  setPopulationYear,
  landPriceYear,
  setLandPriceYear,
  landPriceFeatureCount,
}: LayerRendererProps) {
  return (
    <>
      <BoundaryLayer />
      <AreaHighlight />

      <LandPriceExtrusionLayer
        data={landPriceData}
        visible={visibleLayers.has("land_price_ts")}
        isFetching={isLandPriceFetching}
      />

      {apiLayers.map((layer) => {
        const Component = API_LAYER_COMPONENTS[layer.id];
        if (!Component) return null;
        const layerData =
          areaData != null
            ? ((areaData as Record<string, unknown>)[layer.id] as
                | FeatureCollection
                | undefined)
            : undefined;
        return (
          <Component
            key={layer.id}
            data={layerData ?? EMPTY_FC}
            visible={visibleLayers.has(layer.id)}
          />
        );
      })}

      {staticLayers.map((layer) => {
        if (layer.id === "population_mesh") {
          return (
            <PopulationMeshLayer
              key={layer.id}
              visible={visibleLayers.has(layer.id)}
              selectedYear={populationYear}
            />
          );
        }
        const Component = STATIC_LAYER_COMPONENTS[layer.id];
        if (!Component) return null;
        return (
          <Component key={layer.id} visible={visibleLayers.has(layer.id)} />
        );
      })}

      <YearSlider
        value={populationYear}
        onChange={setPopulationYear}
        visible={visibleLayers.has("population_mesh")}
      />

      <LandPriceYearSlider
        value={landPriceYear}
        onChange={setLandPriceYear}
        visible={visibleLayers.has("land_price_ts")}
        isFetching={isLandPriceFetching}
        isError={isLandPriceError}
        isZoomTooLow={isZoomTooLow}
        {...(landPriceFeatureCount !== undefined
          ? { featureCount: landPriceFeatureCount }
          : {})}
      />
    </>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add services/frontend/src/components/map/layer-renderer.tsx
git commit -m "feat(map): extract LayerRenderer from page.tsx

Renders all 24 layers (static + API), year sliders, and boundary
overlays. Receives data as props — no store access."
```

---

## Task 7: Rewrite page.tsx as Layout Shell

**Files:**
- Rewrite: `services/frontend/src/app/page.tsx`

- [ ] **Step 1: Rewrite page.tsx**

```typescript
// services/frontend/src/app/page.tsx
"use client";

import { ContextPanel } from "@/components/context-panel/context-panel";
import { LayerRenderer } from "@/components/map/layer-renderer";
import { MapView } from "@/components/map/map-view";
import { PopupCard } from "@/components/map/popup-card";
import { StatusBar } from "@/components/status-bar";
import { TopBar } from "@/components/top-bar/top-bar";
import { ExplorePanel } from "@/components/context-panel/explore-panel";
import { ComparePanel } from "@/components/context-panel/compare-panel";
import { useMapInteraction } from "@/hooks/use-map-interaction";
import { useMapPage } from "@/hooks/use-map-page";
import { useUIStore } from "@/stores/ui-store";
import {
  PANEL_WIDTH,
  TOP_BAR_HEIGHT,
  STATUS_BAR_HEIGHT,
} from "@/lib/constants";

export default function Home() {
  const mode = useUIStore((s) => s.mode);
  const { handleFeatureClick } = useMapInteraction();
  const page = useMapPage();

  return (
    <div className="relative h-screen w-screen overflow-hidden">
      <TopBar />

      <ContextPanel>
        {mode === "explore" && <ExplorePanel />}
        {mode === "compare" && <ComparePanel />}
      </ContextPanel>

      <div
        className="absolute"
        style={{
          top: TOP_BAR_HEIGHT,
          left: PANEL_WIDTH,
          right: 0,
          bottom: STATUS_BAR_HEIGHT,
        }}
      >
        <MapView
          onMoveEnd={page.handleMoveEnd}
          onFeatureClick={handleFeatureClick}
        >
          <LayerRenderer
            visibleLayers={page.visibleLayers}
            staticLayers={page.staticLayers}
            apiLayers={page.apiLayers}
            areaData={page.areaData as Record<string, unknown> | null}
            landPriceData={page.landPriceData}
            isLandPriceFetching={page.isLandPriceFetching}
            isLandPriceError={page.isLandPriceError}
            isZoomTooLow={page.isZoomTooLow}
            populationYear={page.populationYear}
            setPopulationYear={page.setPopulationYear}
            landPriceYear={page.landPriceYear}
            setLandPriceYear={page.setLandPriceYear}
            landPriceFeatureCount={page.landPriceData.features.length}
          />
        </MapView>
      </div>

      {/* Spatial popup — currently center-fixed, will move to MapLibre Popup in P1.6 */}
      {page.selectedFeature && page.selectedLayerConfig?.popupFields && (
        <div
          className="fixed z-30 pointer-events-none"
          style={{ top: "50%", left: "50%", transform: "translate(-50%, -50%)" }}
        >
          <div className="pointer-events-auto">
            <PopupCard
              layerNameJa={page.selectedLayerConfig.nameJa}
              fields={page.selectedLayerConfig.popupFields}
              properties={page.selectedFeature.properties}
            />
          </div>
        </div>
      )}

      <StatusBar
        lat={page.viewState.latitude}
        lng={page.viewState.longitude}
        zoom={page.viewState.zoom}
        isLoading={page.isLoading}
        isDemoMode={page.isDemoMode}
        truncatedLayers={page.truncatedLayers}
      />
    </div>
  );
}
```

Key changes from original:
- **ScoreCard removed** — was marked "temporary", replaced by feature detail in Explore panel
- **AnalyzePanel removed** — mode merged into Explore
- **No cross-store mutation** — `useMapInteraction` handles everything within MapStore
- **Layout constants** — no magic numbers
- **~70 lines** vs original 337

- [ ] **Step 2: Verify TypeScript compiles**

```bash
cd services/frontend && pnpm tsc --noEmit
```

- [ ] **Step 3: Run tests**

```bash
cd services/frontend && pnpm vitest run
```

- [ ] **Step 4: Commit**

```bash
git add services/frontend/src/app/page.tsx
git commit -m "refactor(page): rewrite as layout shell (~70 lines)

Extracts useMapPage (data), useMapInteraction (clicks),
LayerRenderer (map layers). Removes ScoreCard (temporary code).
Removes analyze mode panel. Uses layout constants.

337 lines → ~70 lines. No functional regressions."
```

---

## Task 8: Verify Full Build

- [ ] **Step 1: TypeScript check**

```bash
cd services/frontend && pnpm tsc --noEmit
```

- [ ] **Step 2: Biome lint**

```bash
cd services/frontend && pnpm biome check .
```

- [ ] **Step 3: Tests**

```bash
cd services/frontend && pnpm vitest run
```

- [ ] **Step 4: Fix any issues found, then commit**

```bash
git add -A services/frontend/
git commit -m "chore: fix lint/type issues from page.tsx refactor"
```

---

## Post-Plan Notes

**What this plan does NOT do (deferred to subsequent plans):**
- Spatial popup migration to MapLibre Popup (P1.6 step 6)
- Guided first experience (P1.6 step 5)
- Theme card redesign (P1.6 step 7)
- Smooth transitions with framer-motion (P1.6 step 8)
- Design system token implementation (P1.6 step 4)
- Component tests (bundled with each subsequent P1.6 step)

**P0.5 items addressed by this plan:**
- X-04 partial: `selectArea` is now in unified MapStore, ready for boundary click wiring
- X-06 partial: `getBBox` stays as-is for now but is isolated in `useMapPage` for future replacement with `map.getBounds()`
- WASM init error handling added (was silent failure, now logs + sets `wasmError` state)
