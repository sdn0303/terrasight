# Frontend UI/UX Full-Scope Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the complete Phase 1 frontend for the real estate investment data visualization platform — 3D dark map with 6 GeoJSON layers, investment scoring panel, area statistics dashboard, price trend sparklines, and CRT aesthetic.

**Architecture:** Next.js 16 App Router with Server Components by default. MapLibre GL JS renders 6 spatial layers on a CARTO Dark Matter basemap. TanStack Query manages all server state with `ky` as HTTP client. Zustand holds UI state (visible layers, selected feature, compare mode). URL state via `nuqs` enables shareable views. All API responses validated with Zod schemas at the boundary.

**Tech Stack:** Next.js 16, React 19, MapLibre GL JS, react-map-gl, TanStack Query v5, Zustand, nuqs, ky, Zod, React Hook Form, shadcn/ui, Tailwind CSS v4, Recharts, framer-motion, Geist Mono, Vitest + Testing Library

---

## Existing State

The frontend scaffold already contains:
- `src/app/layout.tsx` — Root layout with Geist Mono font
- `src/app/globals.css` — CRT dark theme CSS variables (all design tokens)
- `src/app/page.tsx` — Placeholder "INITIALIZING..." text
- `src/components/crt-overlay.tsx` — Vignette + scanlines (complete)
- `src/components/status-bar.tsx` — Coord/zoom/demo/loading display (complete)
- `src/lib/api.ts` — Basic `fetchApi` wrapper (to be replaced with ky + Zod)
- `src/lib/layers.ts` — Layer config definitions (complete)
- `src/lib/constants.ts` — Map config constants (complete)

**Backend endpoints (fully implemented):**
- `GET /api/health` — Service health check
- `GET /api/area-data?south=&west=&north=&east=&layers=` — GeoJSON layers for bbox
- `GET /api/score?lat=&lng=` — Investment score (0-100) for coordinate
- `GET /api/stats?south=&west=&north=&east=` — Aggregated area statistics
- `GET /api/trend?lat=&lng=&years=` — Price trend / CAGR

**Reference docs:**
- `docs/UIUX_SPEC.md` — Visual spec, component wireframes, interaction spec
- `docs/REQUIREMENTS.md` — FR-1 through FR-8 (Phase 1 scope)
- `docs/API_SPEC.md` — Full request/response contracts
- `.claude/agents/frontend-developer.md` — Agent spec with architecture patterns

---

## Task Dependency Graph

```
Task 1: Dependencies + shadcn/ui init
  ↓
Task 2: Zod schemas + ky API client
  ↓
Task 3: Zustand stores (map, UI state)
  ↓
Task 4: MapView component (react-map-gl)
  ↓
Task 5: GeoJSON layers (6 layer types)
  ↓
Task 6: LayerPanel (left sidebar)
  ↓
Task 7: TanStack Query hooks + data fetching
  ↓
Task 8: ScoreCard (right panel) + investment gauge
  ↓
Task 9: Sparkline (price trend chart)
  ↓
Task 10: DashboardStats (bottom bar)
  ↓
Task 11: URL state sync (nuqs)
  ↓
Task 12: ComparePanel (2-point comparison)
  ↓
Task 13: Error boundaries + loading states
  ↓
Task 14: Responsive layout + polish
```

---

## Task 1: Install Dependencies + Initialize shadcn/ui

**Files:**
- Modify: `services/frontend/package.json`
- Modify: `services/frontend/next.config.ts`
- Create: `services/frontend/src/lib/utils.ts` (shadcn cn() utility)
- Create: `services/frontend/components.json` (shadcn config)

**Step 1: Install runtime dependencies**

```bash
cd services/frontend
pnpm add ky @tanstack/react-query zustand nuqs zod react-hook-form @hookform/resolvers
```

**Step 2: Initialize shadcn/ui**

```bash
cd services/frontend
pnpm dlx shadcn@latest init
```

Choose these options when prompted:
- Style: `new-york`
- Base color: `zinc`
- CSS variables: `yes`

> **Note:** shadcn will modify `globals.css` and `tailwind.config.ts`. After init, restore the CRT design tokens in `globals.css` if overwritten. The existing CSS variables (`--bg-primary`, `--accent-cyan`, etc.) must be preserved.

**Step 3: Add required shadcn/ui components**

```bash
cd services/frontend
pnpm dlx shadcn@latest add button card toggle tooltip scroll-area separator skeleton collapsible sheet
```

**Step 4: Create TanStack Query provider**

Create `services/frontend/src/components/providers.tsx`:

```tsx
"use client";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";

export function Providers({ children }: { children: ReactNode }) {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            staleTime: 60_000,
            gcTime: 300_000,
            retry: 1,
            refetchOnWindowFocus: false,
          },
        },
      }),
  );

  return (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
}
```

**Step 5: Wrap layout with providers**

Modify `services/frontend/src/app/layout.tsx`:

```tsx
import { GeistMono } from "geist/font/mono";
import type { Metadata } from "next";
import { Providers } from "@/components/providers";
import { TooltipProvider } from "@/components/ui/tooltip";
import "./globals.css";

export const metadata: Metadata = {
  title: "RealEstate Intelligence",
  description: "不動産投資意思決定プラットフォーム",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="ja" className={`dark ${GeistMono.variable}`}>
      <body>
        <Providers>
          <TooltipProvider>{children}</TooltipProvider>
        </Providers>
      </body>
    </html>
  );
}
```

**Step 6: Verify build**

```bash
cd services/frontend && pnpm tsc --noEmit
```
Expected: No errors

**Step 7: Commit**

```bash
git add services/frontend/
git commit -m "feat(frontend): install dependencies and initialize shadcn/ui"
```

---

## Task 2: Zod Schemas + ky API Client

**Files:**
- Create: `services/frontend/src/lib/schemas.ts`
- Modify: `services/frontend/src/lib/api.ts`
- Create: `services/frontend/src/__tests__/schemas.test.ts`

**Step 1: Write Zod schemas for all API responses**

Create `services/frontend/src/lib/schemas.ts`:

```typescript
import { z } from "zod";

// ─── GeoJSON primitives ───────────────────────────────
const PointGeometry = z.object({
  type: z.literal("Point"),
  coordinates: z.tuple([z.number(), z.number()]), // [lng, lat]
});

const MultiPolygonGeometry = z.object({
  type: z.literal("MultiPolygon"),
  coordinates: z.array(z.array(z.array(z.tuple([z.number(), z.number()])))),
});

const Geometry = z.discriminatedUnion("type", [
  PointGeometry,
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
  detail: z.record(z.unknown()),
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
  zoning_distribution: z.record(z.number()),
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
```

**Step 2: Write tests for schemas**

Create `services/frontend/src/__tests__/schemas.test.ts`:

```typescript
import { describe, expect, it } from "vitest";
import { HealthResponse, ScoreResponse, StatsResponse, TrendResponse } from "@/lib/schemas";

describe("HealthResponse schema", () => {
  it("parses valid health response", () => {
    const data = {
      status: "ok",
      db_connected: true,
      reinfolib_key_set: false,
      version: "0.1.0",
    };
    expect(HealthResponse.parse(data)).toEqual(data);
  });

  it("rejects invalid status", () => {
    const data = { status: "broken", db_connected: true, reinfolib_key_set: false, version: "0.1.0" };
    expect(() => HealthResponse.parse(data)).toThrow();
  });
});

describe("ScoreResponse schema", () => {
  it("parses valid score response", () => {
    const data = {
      score: 72,
      components: {
        trend: { value: 18, max: 25, detail: { cagr_5y: 0.032 } },
        risk: { value: 22, max: 25, detail: { flood_overlap: 0.0 } },
        access: { value: 15, max: 25, detail: { schools_1km: 3 } },
        yield_potential: { value: 17, max: 25, detail: {} },
      },
      metadata: {
        calculated_at: "2026-03-20T10:30:00Z",
        data_freshness: "2024",
        disclaimer: "本スコアは参考値です。",
      },
    };
    const result = ScoreResponse.parse(data);
    expect(result.score).toBe(72);
    expect(result.components.trend.value).toBe(18);
  });
});

describe("TrendResponse schema", () => {
  it("parses valid trend response", () => {
    const data = {
      location: { address: "千代田区丸の内1", distance_m: 120 },
      data: [
        { year: 2020, price_per_sqm: 1020000 },
        { year: 2024, price_per_sqm: 1200000 },
      ],
      cagr: 0.032,
      direction: "up" as const,
    };
    const result = TrendResponse.parse(data);
    expect(result.direction).toBe("up");
    expect(result.data).toHaveLength(2);
  });
});

describe("StatsResponse schema", () => {
  it("parses valid stats response", () => {
    const data = {
      land_price: { avg_per_sqm: 850000, median_per_sqm: 720000, min_per_sqm: 320000, max_per_sqm: 3200000, count: 45 },
      risk: { flood_area_ratio: 0.15, steep_slope_area_ratio: 0.02, avg_composite_risk: 0.18 },
      facilities: { schools: 12, medical: 28 },
      zoning_distribution: { "商業地域": 0.35, "住居地域": 0.65 },
    };
    const result = StatsResponse.parse(data);
    expect(result.land_price.count).toBe(45);
  });
});
```

**Step 3: Run tests**

```bash
cd services/frontend && pnpm vitest run
```
Expected: All pass

**Step 4: Rewrite API client with ky + Zod validation**

Replace `services/frontend/src/lib/api.ts`:

```typescript
import ky from "ky";
import type { z } from "zod";
import {
  AreaDataResponse,
  HealthResponse,
  ScoreResponse,
  StatsResponse,
  TrendResponse,
} from "./schemas";

const api = ky.create({
  prefixUrl: process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8000",
  timeout: 10_000,
  retry: { limit: 1, statusCodes: [408, 429, 500, 502, 503, 504] },
});

async function get<T>(schema: z.ZodType<T>, path: string, params?: Record<string, string>): Promise<T> {
  const searchParams = params ? new URLSearchParams(params) : undefined;
  const data: unknown = await api.get(path, { searchParams }).json();
  return schema.parse(data);
}

// ─── Typed API functions ──────────────────────────────

export interface BBox {
  south: number;
  west: number;
  north: number;
  east: number;
}

export function fetchHealth() {
  return get(HealthResponse, "api/health");
}

export function fetchAreaData(bbox: BBox, layers: string[]) {
  return get(AreaDataResponse, "api/area-data", {
    south: String(bbox.south),
    west: String(bbox.west),
    north: String(bbox.north),
    east: String(bbox.east),
    layers: layers.join(","),
  });
}

export function fetchScore(lat: number, lng: number) {
  return get(ScoreResponse, "api/score", {
    lat: String(lat),
    lng: String(lng),
  });
}

export function fetchStats(bbox: BBox) {
  return get(StatsResponse, "api/stats", {
    south: String(bbox.south),
    west: String(bbox.west),
    north: String(bbox.north),
    east: String(bbox.east),
  });
}

export function fetchTrend(lat: number, lng: number, years?: number) {
  const params: Record<string, string> = {
    lat: String(lat),
    lng: String(lng),
  };
  if (years !== undefined) {
    params.years = String(years);
  }
  return get(TrendResponse, "api/trend", params);
}
```

**Step 5: Verify build**

```bash
cd services/frontend && pnpm tsc --noEmit && pnpm vitest run
```
Expected: No type errors, all tests pass

**Step 6: Commit**

```bash
git add services/frontend/
git commit -m "feat(frontend): add Zod schemas and ky-based API client with validation"
```

---

## Task 3: Zustand Stores

**Files:**
- Create: `services/frontend/src/stores/map-store.ts`
- Create: `services/frontend/src/stores/ui-store.ts`
- Create: `services/frontend/src/__tests__/map-store.test.ts`

**Step 1: Create map store**

Create `services/frontend/src/stores/map-store.ts`:

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
  coordinates: [number, number]; // [lng, lat]
}

interface MapState {
  // State
  viewState: ViewState;
  visibleLayers: Set<string>;
  selectedFeature: SelectedFeature | null;

  // Actions
  setViewState: (viewState: ViewState) => void;
  toggleLayer: (layerId: string) => void;
  selectFeature: (feature: SelectedFeature | null) => void;
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

      getBBox: () => {
        const { latitude, longitude, zoom } = get().viewState;
        // Approximate bbox from center + zoom
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

**Step 2: Create UI store**

Create `services/frontend/src/stores/ui-store.ts`:

```typescript
import { create } from "zustand";
import { devtools } from "zustand/middleware";

type ComparePoint = {
  lat: number;
  lng: number;
  address: string;
} | null;

interface UIState {
  // Compare mode
  compareMode: boolean;
  comparePointA: ComparePoint;
  comparePointB: ComparePoint;

  // Panel visibility
  layerPanelOpen: boolean;

  // Actions
  enterCompareMode: () => void;
  exitCompareMode: () => void;
  setComparePoint: (point: { lat: number; lng: number; address: string }) => void;
  toggleLayerPanel: () => void;
}

export const useUIStore = create<UIState>()(
  devtools(
    (set, get) => ({
      compareMode: false,
      comparePointA: null,
      comparePointB: null,
      layerPanelOpen: true,

      enterCompareMode: () =>
        set({ compareMode: true, comparePointA: null, comparePointB: null }),

      exitCompareMode: () =>
        set({ compareMode: false, comparePointA: null, comparePointB: null }),

      setComparePoint: (point) => {
        const { comparePointA } = get();
        if (comparePointA === null) {
          set({ comparePointA: point });
        } else {
          set({ comparePointB: point });
        }
      },

      toggleLayerPanel: () =>
        set((state) => ({ layerPanelOpen: !state.layerPanelOpen })),
    }),
    { name: "ui-store" },
  ),
);
```

**Step 3: Write store tests**

Create `services/frontend/src/__tests__/map-store.test.ts`:

```typescript
import { describe, expect, it, beforeEach } from "vitest";
import { useMapStore } from "@/stores/map-store";

describe("useMapStore", () => {
  beforeEach(() => {
    useMapStore.setState({
      visibleLayers: new Set(["landprice", "zoning"]),
      selectedFeature: null,
    });
  });

  it("toggles layer visibility", () => {
    useMapStore.getState().toggleLayer("flood");
    expect(useMapStore.getState().visibleLayers.has("flood")).toBe(true);

    useMapStore.getState().toggleLayer("flood");
    expect(useMapStore.getState().visibleLayers.has("flood")).toBe(false);
  });

  it("selects and deselects feature", () => {
    const feature = {
      layerId: "landprice",
      properties: { id: 1, price_per_sqm: 1200000 },
      coordinates: [139.767, 35.681] as [number, number],
    };
    useMapStore.getState().selectFeature(feature);
    expect(useMapStore.getState().selectedFeature).toEqual(feature);

    useMapStore.getState().selectFeature(null);
    expect(useMapStore.getState().selectedFeature).toBeNull();
  });

  it("default visible layers are landprice and zoning", () => {
    useMapStore.setState({ visibleLayers: new Set(["landprice", "zoning"]) });
    const layers = useMapStore.getState().visibleLayers;
    expect(layers.has("landprice")).toBe(true);
    expect(layers.has("zoning")).toBe(true);
    expect(layers.has("flood")).toBe(false);
  });
});
```

**Step 4: Run tests**

```bash
cd services/frontend && pnpm vitest run
```
Expected: All pass

**Step 5: Commit**

```bash
git add services/frontend/
git commit -m "feat(frontend): add Zustand stores for map and UI state"
```

---

## Task 4: MapView Component

**Files:**
- Create: `services/frontend/src/components/map/map-view.tsx`
- Modify: `services/frontend/src/app/page.tsx`

> **Important:** MapLibre GL requires `maplibre-gl/dist/maplibre-gl.css` imported. The `<Map>` component from `react-map-gl` must be wrapped with a client-side mount guard because MapLibre doesn't work in SSR.

**Step 1: Create MapView component**

Create `services/frontend/src/components/map/map-view.tsx`:

```tsx
"use client";

import "maplibre-gl/dist/maplibre-gl.css";

import { useCallback, useEffect, useState, type ReactNode } from "react";
import { Map, NavigationControl } from "react-map-gl/maplibre";
import type { MapLayerMouseEvent, ViewStateChangeEvent } from "react-map-gl/maplibre";
import { MAP_CONFIG, DEBOUNCE_MS } from "@/lib/constants";
import { useMapStore } from "@/stores/map-store";

interface MapViewProps {
  children?: ReactNode;
  onMoveEnd?: () => void;
  onFeatureClick?: (e: MapLayerMouseEvent) => void;
}

export function MapView({ children, onMoveEnd, onFeatureClick }: MapViewProps) {
  const [mounted, setMounted] = useState(false);
  const { viewState, setViewState } = useMapStore();

  useEffect(() => {
    setMounted(true);
  }, []);

  const handleMove = useCallback(
    (e: ViewStateChangeEvent) => {
      setViewState(e.viewState);
    },
    [setViewState],
  );

  // Debounced move end
  const handleMoveEnd = useCallback(() => {
    const timer = setTimeout(() => {
      onMoveEnd?.();
    }, DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [onMoveEnd]);

  const handleClick = useCallback(
    (e: MapLayerMouseEvent) => {
      onFeatureClick?.(e);
    },
    [onFeatureClick],
  );

  if (!mounted) {
    return (
      <div
        className="flex items-center justify-center"
        style={{
          width: "100%",
          height: "100%",
          background: "var(--bg-primary)",
          color: "var(--accent-cyan)",
          fontFamily: "var(--font-mono)",
          fontSize: "14px",
        }}
      >
        LOADING MAP...
      </div>
    );
  }

  return (
    <Map
      {...viewState}
      onMove={handleMove}
      onMoveEnd={handleMoveEnd}
      onClick={handleClick}
      mapStyle={MAP_CONFIG.style}
      style={{ width: "100%", height: "100%" }}
      attributionControl={false}
    >
      <NavigationControl position="bottom-right" />
      {children}
    </Map>
  );
}
```

**Step 2: Update page.tsx to render MapView**

Replace `services/frontend/src/app/page.tsx`:

```tsx
"use client";

import { MapView } from "@/components/map/map-view";
import { CRTOverlay } from "@/components/crt-overlay";
import { StatusBar } from "@/components/status-bar";
import { useMapStore } from "@/stores/map-store";

export default function Home() {
  const { viewState } = useMapStore();

  return (
    <div className="relative h-screen w-screen overflow-hidden">
      {/* Map */}
      <MapView />

      {/* CRT effect overlays */}
      <CRTOverlay />

      {/* Status bar */}
      <StatusBar
        lat={viewState.latitude}
        lng={viewState.longitude}
        zoom={viewState.zoom}
        isLoading={false}
        isDemoMode={true}
      />
    </div>
  );
}
```

**Step 3: Verify build and manual check**

```bash
cd services/frontend && pnpm tsc --noEmit
```
Expected: No type errors

> Manual verification: Run `pnpm dev` and navigate to http://localhost:3000. You should see a dark 3D map centered on Tokyo Station with CRT overlay and status bar.

**Step 4: Commit**

```bash
git add services/frontend/
git commit -m "feat(frontend): add MapView with react-map-gl and CARTO Dark Matter basemap"
```

---

## Task 5: GeoJSON Map Layers

**Files:**
- Create: `services/frontend/src/components/map/layers/landprice-layer.tsx`
- Create: `services/frontend/src/components/map/layers/zoning-layer.tsx`
- Create: `services/frontend/src/components/map/layers/flood-layer.tsx`
- Create: `services/frontend/src/components/map/layers/steep-slope-layer.tsx`
- Create: `services/frontend/src/components/map/layers/school-layer.tsx`
- Create: `services/frontend/src/components/map/layers/medical-layer.tsx`
- Create: `services/frontend/src/components/map/layers/index.tsx`

Each layer component follows the same pattern: accept GeoJSON data as prop, render a `<Source>` + `<Layer>` from react-map-gl.

> **Reference:** `docs/UIUX_SPEC.md` section 5 has exact paint specifications for each layer type.

**Step 1: Create landprice layer (circle)**

Create `services/frontend/src/components/map/layers/landprice-layer.tsx`:

```tsx
"use client";

import { Layer, Source } from "react-map-gl/maplibre";
import type { FeatureCollection } from "geojson";

interface Props {
  data: FeatureCollection;
  visible: boolean;
}

export function LandpriceLayer({ data, visible }: Props) {
  if (!visible) return null;

  return (
    <Source id="landprice" type="geojson" data={data}>
      <Layer
        id="landprice-circle"
        type="circle"
        paint={{
          "circle-radius": ["interpolate", ["linear"], ["zoom"], 10, 3, 15, 8],
          "circle-color": "#22d3ee",
          "circle-opacity": 0.8,
          "circle-stroke-width": 1,
          "circle-stroke-color": "#0a0a0f",
        }}
      />
    </Source>
  );
}
```

**Step 2: Create zoning layer (fill)**

Create `services/frontend/src/components/map/layers/zoning-layer.tsx`:

```tsx
"use client";

import { Layer, Source } from "react-map-gl/maplibre";
import type { FeatureCollection } from "geojson";

interface Props {
  data: FeatureCollection;
  visible: boolean;
}

export function ZoningLayer({ data, visible }: Props) {
  if (!visible) return null;

  return (
    <Source id="zoning" type="geojson" data={data}>
      <Layer
        id="zoning-fill"
        type="fill"
        paint={{
          "fill-color": [
            "match",
            ["get", "zone_type"],
            "第一種低層住居専用地域", "#2563eb",
            "第二種低層住居専用地域", "#3b82f6",
            "第一種中高層住居専用地域", "#60a5fa",
            "第二種中高層住居専用地域", "#93c5fd",
            "第一種住居地域", "#a78bfa",
            "第二種住居地域", "#c4b5fd",
            "準住居地域", "#e9d5ff",
            "近隣商業地域", "#fbbf24",
            "商業地域", "#f97316",
            "準工業地域", "#a3e635",
            "工業地域", "#6b7280",
            "工業専用地域", "#374151",
            "#6b7280",
          ],
          "fill-opacity": 0.35,
        }}
      />
    </Source>
  );
}
```

**Step 3: Create flood layer (fill-extrusion)**

Create `services/frontend/src/components/map/layers/flood-layer.tsx`:

```tsx
"use client";

import { Layer, Source } from "react-map-gl/maplibre";
import type { FeatureCollection } from "geojson";

interface Props {
  data: FeatureCollection;
  visible: boolean;
}

export function FloodLayer({ data, visible }: Props) {
  if (!visible) return null;

  return (
    <Source id="flood" type="geojson" data={data}>
      <Layer
        id="flood-extrusion"
        type="fill-extrusion"
        paint={{
          "fill-extrusion-color": [
            "interpolate",
            ["linear"],
            ["get", "depth_rank"],
            0, "#1a6fff",
            2, "#ffd000",
            4, "#e04030",
          ],
          "fill-extrusion-height": ["*", ["get", "depth_rank"], 50],
          "fill-extrusion-base": 0,
          "fill-extrusion-opacity": 0.7,
        }}
      />
    </Source>
  );
}
```

**Step 4: Create steep slope layer (fill-extrusion)**

Create `services/frontend/src/components/map/layers/steep-slope-layer.tsx`:

```tsx
"use client";

import { Layer, Source } from "react-map-gl/maplibre";
import type { FeatureCollection } from "geojson";

interface Props {
  data: FeatureCollection;
  visible: boolean;
}

export function SteepSlopeLayer({ data, visible }: Props) {
  if (!visible) return null;

  return (
    <Source id="steep_slope" type="geojson" data={data}>
      <Layer
        id="steep-slope-extrusion"
        type="fill-extrusion"
        paint={{
          "fill-extrusion-color": "#e04030",
          "fill-extrusion-height": 100,
          "fill-extrusion-base": 0,
          "fill-extrusion-opacity": 0.6,
        }}
      />
    </Source>
  );
}
```

**Step 5: Create school layer (circle)**

Create `services/frontend/src/components/map/layers/school-layer.tsx`:

```tsx
"use client";

import { Layer, Source } from "react-map-gl/maplibre";
import type { FeatureCollection } from "geojson";

interface Props {
  data: FeatureCollection;
  visible: boolean;
}

export function SchoolLayer({ data, visible }: Props) {
  if (!visible) return null;

  return (
    <Source id="schools" type="geojson" data={data}>
      <Layer
        id="schools-circle"
        type="circle"
        paint={{
          "circle-radius": 5,
          "circle-color": "#10b981",
          "circle-opacity": 0.9,
        }}
      />
    </Source>
  );
}
```

**Step 6: Create medical layer (circle)**

Create `services/frontend/src/components/map/layers/medical-layer.tsx`:

```tsx
"use client";

import { Layer, Source } from "react-map-gl/maplibre";
import type { FeatureCollection } from "geojson";

interface Props {
  data: FeatureCollection;
  visible: boolean;
}

export function MedicalLayer({ data, visible }: Props) {
  if (!visible) return null;

  return (
    <Source id="medical" type="geojson" data={data}>
      <Layer
        id="medical-circle"
        type="circle"
        paint={{
          "circle-radius": 5,
          "circle-color": "#6ee7b7",
          "circle-opacity": 0.9,
        }}
      />
    </Source>
  );
}
```

**Step 7: Create barrel export**

Create `services/frontend/src/components/map/layers/index.tsx`:

```tsx
export { LandpriceLayer } from "./landprice-layer";
export { ZoningLayer } from "./zoning-layer";
export { FloodLayer } from "./flood-layer";
export { SteepSlopeLayer } from "./steep-slope-layer";
export { SchoolLayer } from "./school-layer";
export { MedicalLayer } from "./medical-layer";
```

**Step 8: Verify build**

```bash
cd services/frontend && pnpm tsc --noEmit
```
Expected: No type errors

**Step 9: Commit**

```bash
git add services/frontend/
git commit -m "feat(frontend): add 6 GeoJSON map layer components"
```

---

## Task 6: LayerPanel (Left Sidebar)

**Files:**
- Create: `services/frontend/src/components/layer-panel.tsx`

> **Reference:** `docs/UIUX_SPEC.md` section 4.2 — LayerPanel wireframe with category groups and toggle states.

**Step 1: Create LayerPanel**

Create `services/frontend/src/components/layer-panel.tsx`:

```tsx
"use client";

import { motion, AnimatePresence } from "framer-motion";
import { LAYERS, CATEGORIES } from "@/lib/layers";
import { useMapStore } from "@/stores/map-store";
import { useUIStore } from "@/stores/ui-store";

export function LayerPanel() {
  const { visibleLayers, toggleLayer } = useMapStore();
  const { layerPanelOpen } = useUIStore();

  return (
    <AnimatePresence>
      {layerPanelOpen && (
        <motion.aside
          initial={{ x: -280 }}
          animate={{ x: 0 }}
          exit={{ x: -280 }}
          transition={{ duration: 0.3 }}
          className="fixed left-0 top-0 bottom-[28px] overflow-y-auto"
          style={{
            width: 280,
            background: "var(--bg-secondary)",
            borderRight: "1px solid var(--border-primary)",
            zIndex: 40,
          }}
          aria-label="Layer controls"
        >
          {/* Header */}
          <div className="px-4 pt-4 pb-2">
            <div
              className="text-xs tracking-[0.15em]"
              style={{ color: "var(--accent-cyan)" }}
            >
              ▸ REALESTATE
            </div>
            <div
              className="text-xs tracking-[0.15em]"
              style={{ color: "var(--accent-cyan)" }}
            >
              &nbsp; INTELLIGENCE
            </div>
          </div>

          {/* Layer groups */}
          {CATEGORIES.map((category) => {
            const categoryLayers = LAYERS.filter(
              (l) => l.category === category.id,
            );
            return (
              <div key={category.id} className="px-4 py-2">
                <div
                  className="text-[9px] tracking-[0.15em] mb-2"
                  style={{ color: "var(--text-muted)" }}
                >
                  ── {category.label} ──
                </div>
                {categoryLayers.map((layer) => {
                  const isActive = visibleLayers.has(layer.id);
                  return (
                    <button
                      key={layer.id}
                      type="button"
                      onClick={() => toggleLayer(layer.id)}
                      className="flex items-center gap-2 w-full px-2 py-1.5 rounded text-left text-xs transition-colors"
                      style={{
                        background: isActive
                          ? "var(--hover-accent)"
                          : "transparent",
                        color: isActive
                          ? "var(--text-primary)"
                          : "var(--text-muted)",
                      }}
                      aria-pressed={isActive}
                      aria-label={`${layer.nameJa} layer toggle`}
                    >
                      <span
                        className="inline-block w-2 h-2 rounded-full"
                        style={{
                          background: isActive
                            ? "var(--accent-cyan)"
                            : "var(--text-muted)",
                        }}
                      />
                      {layer.nameJa}
                    </button>
                  );
                })}
              </div>
            );
          })}
        </motion.aside>
      )}
    </AnimatePresence>
  );
}
```

**Step 2: Verify build**

```bash
cd services/frontend && pnpm tsc --noEmit
```

**Step 3: Commit**

```bash
git add services/frontend/
git commit -m "feat(frontend): add LayerPanel sidebar with category groups and toggles"
```

---

## Task 7: TanStack Query Hooks + Data Fetching Integration

**Files:**
- Create: `services/frontend/src/features/area-data/api/use-area-data.ts`
- Create: `services/frontend/src/features/score/api/use-score.ts`
- Create: `services/frontend/src/features/stats/api/use-stats.ts`
- Create: `services/frontend/src/features/trend/api/use-trend.ts`
- Create: `services/frontend/src/features/health/api/use-health.ts`
- Create: `services/frontend/src/lib/query-keys.ts`
- Modify: `services/frontend/src/app/page.tsx` — wire up data fetching

**Step 1: Create query key factory**

Create `services/frontend/src/lib/query-keys.ts`:

```typescript
import type { BBox } from "./api";

export const queryKeys = {
  health: ["health"] as const,

  areaData: {
    all: ["area-data"] as const,
    bbox: (bbox: BBox, layers: string[]) =>
      ["area-data", bbox, layers.sort().join(",")] as const,
  },

  score: {
    all: ["score"] as const,
    coord: (lat: number, lng: number) => ["score", lat, lng] as const,
  },

  stats: {
    all: ["stats"] as const,
    bbox: (bbox: BBox) => ["stats", bbox] as const,
  },

  trend: {
    all: ["trend"] as const,
    coord: (lat: number, lng: number, years?: number) =>
      ["trend", lat, lng, years] as const,
  },
};
```

**Step 2: Create area data hook**

Create `services/frontend/src/features/area-data/api/use-area-data.ts`:

```typescript
import { useQuery } from "@tanstack/react-query";
import { fetchAreaData, type BBox } from "@/lib/api";
import { queryKeys } from "@/lib/query-keys";

export function useAreaData(bbox: BBox | null, layers: string[]) {
  return useQuery({
    queryKey: queryKeys.areaData.bbox(bbox ?? { south: 0, west: 0, north: 0, east: 0 }, layers),
    queryFn: () => fetchAreaData(bbox!, layers),
    enabled: bbox !== null && layers.length > 0,
    staleTime: 60_000,
  });
}
```

**Step 3: Create score hook**

Create `services/frontend/src/features/score/api/use-score.ts`:

```typescript
import { useQuery } from "@tanstack/react-query";
import { fetchScore } from "@/lib/api";
import { queryKeys } from "@/lib/query-keys";

export function useScore(lat: number | null, lng: number | null) {
  return useQuery({
    queryKey: queryKeys.score.coord(lat ?? 0, lng ?? 0),
    queryFn: () => fetchScore(lat!, lng!),
    enabled: lat !== null && lng !== null,
  });
}
```

**Step 4: Create stats hook**

Create `services/frontend/src/features/stats/api/use-stats.ts`:

```typescript
import { useQuery } from "@tanstack/react-query";
import { fetchStats, type BBox } from "@/lib/api";
import { queryKeys } from "@/lib/query-keys";

export function useStats(bbox: BBox | null) {
  return useQuery({
    queryKey: queryKeys.stats.bbox(bbox ?? { south: 0, west: 0, north: 0, east: 0 }),
    queryFn: () => fetchStats(bbox!),
    enabled: bbox !== null,
  });
}
```

**Step 5: Create trend hook**

Create `services/frontend/src/features/trend/api/use-trend.ts`:

```typescript
import { useQuery } from "@tanstack/react-query";
import { fetchTrend } from "@/lib/api";
import { queryKeys } from "@/lib/query-keys";

export function useTrend(lat: number | null, lng: number | null, years?: number) {
  return useQuery({
    queryKey: queryKeys.trend.coord(lat ?? 0, lng ?? 0, years),
    queryFn: () => fetchTrend(lat!, lng!, years),
    enabled: lat !== null && lng !== null,
  });
}
```

**Step 6: Create health hook**

Create `services/frontend/src/features/health/api/use-health.ts`:

```typescript
import { useQuery } from "@tanstack/react-query";
import { fetchHealth } from "@/lib/api";
import { queryKeys } from "@/lib/query-keys";

export function useHealth() {
  return useQuery({
    queryKey: queryKeys.health,
    queryFn: fetchHealth,
    staleTime: 30_000,
  });
}
```

**Step 7: Wire up page.tsx with data fetching + layers**

Replace `services/frontend/src/app/page.tsx`:

```tsx
"use client";

import { useCallback, useMemo, useRef } from "react";
import type { MapLayerMouseEvent } from "react-map-gl/maplibre";
import { MapView } from "@/components/map/map-view";
import {
  LandpriceLayer,
  ZoningLayer,
  FloodLayer,
  SteepSlopeLayer,
  SchoolLayer,
  MedicalLayer,
} from "@/components/map/layers";
import { CRTOverlay } from "@/components/crt-overlay";
import { StatusBar } from "@/components/status-bar";
import { LayerPanel } from "@/components/layer-panel";
import { useMapStore } from "@/stores/map-store";
import { useAreaData } from "@/features/area-data/api/use-area-data";
import { useHealth } from "@/features/health/api/use-health";

const EMPTY_FC: GeoJSON.FeatureCollection = { type: "FeatureCollection", features: [] };

export default function Home() {
  const { viewState, visibleLayers, selectFeature, getBBox } = useMapStore();
  const bboxRef = useRef(getBBox());

  const layers = useMemo(() => [...visibleLayers], [visibleLayers]);
  const { data: areaData, isLoading } = useAreaData(bboxRef.current, layers);
  const { data: health } = useHealth();

  const handleMoveEnd = useCallback(() => {
    bboxRef.current = getBBox();
  }, [getBBox]);

  const handleFeatureClick = useCallback(
    (e: MapLayerMouseEvent) => {
      const feature = e.features?.[0];
      if (feature) {
        selectFeature({
          layerId: feature.layer.id,
          properties: feature.properties as Record<string, unknown>,
          coordinates: [e.lngLat.lng, e.lngLat.lat],
        });
      } else {
        selectFeature(null);
      }
    },
    [selectFeature],
  );

  const isDemoMode = health ? !health.reinfolib_key_set : true;

  return (
    <div className="relative h-screen w-screen overflow-hidden">
      {/* Left panel */}
      <LayerPanel />

      {/* Map with layers */}
      <MapView onMoveEnd={handleMoveEnd} onFeatureClick={handleFeatureClick}>
        <LandpriceLayer
          data={areaData?.landprice ?? EMPTY_FC}
          visible={visibleLayers.has("landprice")}
        />
        <ZoningLayer
          data={areaData?.zoning ?? EMPTY_FC}
          visible={visibleLayers.has("zoning")}
        />
        <FloodLayer
          data={areaData?.flood ?? EMPTY_FC}
          visible={visibleLayers.has("flood")}
        />
        <SteepSlopeLayer
          data={areaData?.steep_slope ?? EMPTY_FC}
          visible={visibleLayers.has("steep_slope")}
        />
        <SchoolLayer
          data={areaData?.schools ?? EMPTY_FC}
          visible={visibleLayers.has("schools")}
        />
        <MedicalLayer
          data={areaData?.medical ?? EMPTY_FC}
          visible={visibleLayers.has("medical")}
        />
      </MapView>

      {/* Overlays */}
      <CRTOverlay />
      <StatusBar
        lat={viewState.latitude}
        lng={viewState.longitude}
        zoom={viewState.zoom}
        isLoading={isLoading}
        isDemoMode={isDemoMode}
      />
    </div>
  );
}
```

**Step 8: Verify build**

```bash
cd services/frontend && pnpm tsc --noEmit && pnpm vitest run
```

**Step 9: Commit**

```bash
git add services/frontend/
git commit -m "feat(frontend): add TanStack Query hooks and wire up map data fetching"
```

---

## Task 8: ScoreCard (Right Panel)

**Files:**
- Create: `services/frontend/src/components/score-card/score-card.tsx`
- Create: `services/frontend/src/components/score-card/score-gauge.tsx`
- Create: `services/frontend/src/components/score-card/component-bar.tsx`
- Modify: `services/frontend/src/app/page.tsx`

> **Reference:** `docs/UIUX_SPEC.md` section 4.3 — ScoreCard wireframe with investment gauge, pricing, disaster risk, and facilities.

**Step 1: Create SVG score gauge**

Create `services/frontend/src/components/score-card/score-gauge.tsx`:

```tsx
"use client";

interface ScoreGaugeProps {
  score: number; // 0-100
}

function getScoreColor(score: number): string {
  if (score < 34) return "var(--accent-danger)";
  if (score < 67) return "var(--accent-warning)";
  return "var(--accent-success)";
}

export function ScoreGauge({ score }: ScoreGaugeProps) {
  const color = getScoreColor(score);
  const clampedScore = Math.max(0, Math.min(100, score));
  // SVG arc: 180 degrees = score 100
  const angle = (clampedScore / 100) * 180;
  const rad = (angle * Math.PI) / 180;
  const r = 60;
  const cx = 80;
  const cy = 75;
  const x = cx + r * Math.cos(Math.PI - rad);
  const y = cy - r * Math.sin(Math.PI - rad);
  const largeArc = angle > 90 ? 1 : 0;

  return (
    <svg viewBox="0 0 160 100" className="w-full" aria-label={`Investment score: ${score}`}>
      {/* Background arc */}
      <path
        d={`M ${cx - r} ${cy} A ${r} ${r} 0 0 1 ${cx + r} ${cy}`}
        fill="none"
        stroke="var(--bg-tertiary)"
        strokeWidth={8}
      />
      {/* Score arc */}
      {clampedScore > 0 && (
        <path
          d={`M ${cx - r} ${cy} A ${r} ${r} 0 ${largeArc} 1 ${x} ${y}`}
          fill="none"
          stroke={color}
          strokeWidth={8}
          strokeLinecap="round"
        />
      )}
      {/* Score text */}
      <text
        x={cx}
        y={cy - 10}
        textAnchor="middle"
        fill={color}
        fontSize={28}
        fontFamily="var(--font-mono)"
        fontWeight="bold"
      >
        {Math.round(score)}
      </text>
      {/* Labels */}
      <text x={cx - r} y={cy + 15} textAnchor="middle" fill="var(--text-muted)" fontSize={9}>
        0
      </text>
      <text x={cx} y={cy + 15} textAnchor="middle" fill="var(--text-muted)" fontSize={9}>
        50
      </text>
      <text x={cx + r} y={cy + 15} textAnchor="middle" fill="var(--text-muted)" fontSize={9}>
        100
      </text>
    </svg>
  );
}
```

**Step 2: Create component bar**

Create `services/frontend/src/components/score-card/component-bar.tsx`:

```tsx
interface ComponentBarProps {
  label: string;
  value: number;
  max: number;
}

export function ComponentBar({ label, value, max }: ComponentBarProps) {
  const pct = max > 0 ? (value / max) * 100 : 0;
  return (
    <div className="flex items-center gap-2 text-[10px]" style={{ fontFamily: "var(--font-mono)" }}>
      <span className="w-16 text-right" style={{ color: "var(--text-muted)" }}>
        {label}:
      </span>
      <div className="flex-1 h-1.5 rounded" style={{ background: "var(--bg-tertiary)" }}>
        <div
          className="h-full rounded"
          style={{
            width: `${pct}%`,
            background: "var(--accent-cyan)",
          }}
        />
      </div>
      <span style={{ color: "var(--text-secondary)" }}>
        {value}/{max}
      </span>
    </div>
  );
}
```

**Step 3: Create ScoreCard component**

Create `services/frontend/src/components/score-card/score-card.tsx`:

```tsx
"use client";

import { motion, AnimatePresence } from "framer-motion";
import { ScoreGauge } from "./score-gauge";
import { ComponentBar } from "./component-bar";
import { useScore } from "@/features/score/api/use-score";
import { useMapStore } from "@/stores/map-store";
import { Skeleton } from "@/components/ui/skeleton";

export function ScoreCard() {
  const { selectedFeature, selectFeature } = useMapStore();
  const lat = selectedFeature?.coordinates[1] ?? null;
  const lng = selectedFeature?.coordinates[0] ?? null;
  const { data: score, isLoading } = useScore(lat, lng);

  return (
    <AnimatePresence>
      {selectedFeature && (
        <motion.aside
          initial={{ x: 320 }}
          animate={{ x: 0 }}
          exit={{ x: 320 }}
          transition={{ duration: 0.3 }}
          className="fixed right-4 top-4 bottom-[148px] overflow-y-auto rounded-lg"
          style={{
            width: 320,
            background: "rgba(10, 10, 15, 0.9)",
            backdropFilter: "blur(12px)",
            border: "1px solid var(--border-primary)",
            zIndex: 50,
          }}
          aria-label="Property score card"
        >
          {/* Header */}
          <div className="flex items-center justify-between px-4 py-3 border-b" style={{ borderColor: "var(--border-primary)" }}>
            <span className="text-[9px] tracking-[0.15em]" style={{ color: "var(--accent-cyan)" }}>
              PROPERTY INTEL
            </span>
            <button
              type="button"
              onClick={() => selectFeature(null)}
              className="text-xs"
              style={{ color: "var(--text-muted)" }}
              aria-label="Close score card"
            >
              ×
            </button>
          </div>

          <div className="p-4 space-y-4">
            {/* Location */}
            <div>
              <div className="text-[9px] tracking-[0.15em] mb-1" style={{ color: "var(--text-muted)" }}>
                LOCATION
              </div>
              <div className="text-xs" style={{ color: "var(--text-primary)" }}>
                {String(selectedFeature.properties.address ?? `${lat?.toFixed(4)}°N, ${lng?.toFixed(4)}°E`)}
              </div>
            </div>

            {/* Investment Score */}
            {isLoading ? (
              <div className="space-y-2">
                <Skeleton className="h-4 w-32" />
                <Skeleton className="h-24 w-full" />
              </div>
            ) : score ? (
              <div className="rounded-lg p-3" style={{ background: "var(--bg-tertiary)" }}>
                <div className="text-[9px] tracking-[0.15em] mb-2" style={{ color: "var(--text-muted)" }}>
                  INVESTMENT SCORE
                </div>
                <ScoreGauge score={score.score} />
                <div className="space-y-1.5 mt-3">
                  <ComponentBar label="trend" value={score.components.trend.value} max={score.components.trend.max} />
                  <ComponentBar label="risk" value={score.components.risk.value} max={score.components.risk.max} />
                  <ComponentBar label="access" value={score.components.access.value} max={score.components.access.max} />
                  <ComponentBar label="yield" value={score.components.yield_potential.value} max={score.components.yield_potential.max} />
                </div>
              </div>
            ) : null}

            {/* Pricing */}
            {selectedFeature.properties.price_per_sqm !== undefined && (
              <div className="rounded-lg p-3" style={{ background: "var(--bg-tertiary)" }}>
                <div className="text-[9px] tracking-[0.15em] mb-2" style={{ color: "var(--text-muted)" }}>
                  PRICING
                </div>
                <div className="flex justify-between text-xs">
                  <span style={{ color: "var(--text-secondary)" }}>per sqm</span>
                  <span style={{ color: "var(--accent-cyan)" }}>
                    ¥{Number(selectedFeature.properties.price_per_sqm).toLocaleString()}
                  </span>
                </div>
              </div>
            )}

            {/* Disclaimer */}
            {score && (
              <div className="text-[9px]" style={{ color: "var(--text-muted)" }}>
                {score.metadata.disclaimer}
              </div>
            )}
          </div>
        </motion.aside>
      )}
    </AnimatePresence>
  );
}
```

**Step 4: Add ScoreCard to page.tsx**

Add import and component to `services/frontend/src/app/page.tsx` — add `<ScoreCard />` after `<LayerPanel />` inside the container.

**Step 5: Verify build**

```bash
cd services/frontend && pnpm tsc --noEmit
```

**Step 6: Commit**

```bash
git add services/frontend/
git commit -m "feat(frontend): add ScoreCard with investment gauge and component bars"
```

---

## Task 9: Price Trend Sparkline

**Files:**
- Create: `services/frontend/src/components/score-card/sparkline.tsx`
- Modify: `services/frontend/src/components/score-card/score-card.tsx`

> **Reference:** `docs/UIUX_SPEC.md` section 4.3 — PRICE TREND sparkline with CAGR display.

**Step 1: Create Sparkline component using Recharts**

Create `services/frontend/src/components/score-card/sparkline.tsx`:

```tsx
"use client";

import { LineChart, Line, ResponsiveContainer, XAxis, YAxis, Tooltip } from "recharts";
import type { TrendResponse } from "@/lib/schemas";

interface SparklineProps {
  trend: TrendResponse;
}

export function Sparkline({ trend }: SparklineProps) {
  const color = trend.direction === "up" ? "var(--accent-success)" : "var(--accent-danger)";
  const cagrPct = (trend.cagr * 100).toFixed(1);
  const sign = trend.direction === "up" ? "+" : "";

  return (
    <div className="rounded-lg p-3" style={{ background: "var(--bg-tertiary)" }}>
      <div className="text-[9px] tracking-[0.15em] mb-2" style={{ color: "var(--text-muted)" }}>
        PRICE TREND
      </div>
      <div style={{ width: "100%", height: 40 }}>
        <ResponsiveContainer>
          <LineChart data={trend.data}>
            <XAxis dataKey="year" hide />
            <YAxis hide domain={["dataMin", "dataMax"]} />
            <Tooltip
              contentStyle={{
                background: "var(--bg-secondary)",
                border: "1px solid var(--border-primary)",
                fontSize: 10,
                fontFamily: "var(--font-mono)",
              }}
              formatter={(value: number) => [`¥${value.toLocaleString()}`, "per sqm"]}
            />
            <Line
              type="monotone"
              dataKey="price_per_sqm"
              stroke={color}
              strokeWidth={2}
              dot={false}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
      <div className="flex justify-between mt-1 text-[10px]">
        <span style={{ color: "var(--text-muted)" }}>
          {trend.data[0]?.year} — {trend.data[trend.data.length - 1]?.year}
        </span>
        <span style={{ color }}>
          CAGR: {sign}{cagrPct}%
        </span>
      </div>
    </div>
  );
}
```

**Step 2: Add Sparkline to ScoreCard**

Modify `services/frontend/src/components/score-card/score-card.tsx`:
- Import `useTrend` from `@/features/trend/api/use-trend`
- Import `Sparkline` from `./sparkline`
- Call `useTrend(lat, lng)` alongside `useScore`
- Render `<Sparkline trend={trendData} />` after the pricing section when trend data is available

**Step 3: Verify build**

```bash
cd services/frontend && pnpm tsc --noEmit
```

**Step 4: Commit**

```bash
git add services/frontend/
git commit -m "feat(frontend): add price trend sparkline with CAGR display"
```

---

## Task 10: DashboardStats (Bottom Bar)

**Files:**
- Create: `services/frontend/src/components/dashboard-stats.tsx`
- Modify: `services/frontend/src/app/page.tsx`

> **Reference:** `docs/UIUX_SPEC.md` section 4.4 — DashboardStats with 4 stat cards.

**Step 1: Create DashboardStats component**

Create `services/frontend/src/components/dashboard-stats.tsx`:

```tsx
"use client";

import { Skeleton } from "@/components/ui/skeleton";
import { useStats } from "@/features/stats/api/use-stats";
import { useMapStore } from "@/stores/map-store";

function StatCard({
  label,
  value,
  sub,
  color,
}: {
  label: string;
  value: string;
  sub?: string;
  color?: string;
}) {
  return (
    <div
      className="flex-1 rounded-lg p-3"
      style={{ background: "var(--bg-tertiary)" }}
    >
      <div
        className="text-[9px] tracking-[0.15em] mb-1"
        style={{ color: "var(--text-muted)" }}
      >
        {label}
      </div>
      <div
        className="text-lg font-bold"
        style={{ color: color ?? "var(--accent-cyan)" }}
      >
        {value}
      </div>
      {sub && (
        <div className="text-[10px]" style={{ color: "var(--text-secondary)" }}>
          {sub}
        </div>
      )}
    </div>
  );
}

export function DashboardStats() {
  const bbox = useMapStore((s) => s.getBBox());
  const { data: stats, isLoading } = useStats(bbox);

  if (isLoading) {
    return (
      <div
        className="fixed left-0 right-0 flex gap-3 px-4 py-3"
        style={{
          bottom: 28,
          height: 120,
          background: "var(--bg-secondary)",
          borderTop: "1px solid var(--border-primary)",
          zIndex: 30,
        }}
      >
        {Array.from({ length: 4 }).map((_, i) => (
          <div key={i} className="flex-1 rounded-lg p-3" style={{ background: "var(--bg-tertiary)" }}>
            <Skeleton className="h-3 w-16 mb-2" />
            <Skeleton className="h-6 w-24" />
          </div>
        ))}
      </div>
    );
  }

  if (!stats) return null;

  const riskPct = Math.round(stats.risk.avg_composite_risk * 100);

  return (
    <div
      className="fixed left-0 right-0 flex gap-3 px-4 py-3"
      style={{
        bottom: 28,
        height: 120,
        background: "var(--bg-secondary)",
        borderTop: "1px solid var(--border-primary)",
        zIndex: 30,
      }}
      aria-label="Area statistics"
    >
      <StatCard
        label="AVG PRICE"
        value={`¥${stats.land_price.avg_per_sqm.toLocaleString()}`}
        sub={`med: ¥${stats.land_price.median_per_sqm.toLocaleString()}`}
      />
      <StatCard
        label="LISTINGS"
        value={String(stats.land_price.count)}
      />
      <StatCard
        label="RISK"
        value={`${riskPct}%`}
        color={riskPct > 30 ? "var(--accent-danger)" : "var(--accent-success)"}
      />
      <StatCard
        label="FACILITIES"
        value={String(stats.facilities.schools + stats.facilities.medical)}
        sub={`${stats.facilities.schools} schools, ${stats.facilities.medical} medical`}
      />
    </div>
  );
}
```

**Step 2: Add to page.tsx**

Import and add `<DashboardStats />` after `<StatusBar />`.

**Step 3: Verify build**

```bash
cd services/frontend && pnpm tsc --noEmit
```

**Step 4: Commit**

```bash
git add services/frontend/
git commit -m "feat(frontend): add DashboardStats bottom bar with area statistics"
```

---

## Task 11: URL State Sync (nuqs)

**Files:**
- Create: `services/frontend/src/hooks/use-map-url-state.ts`
- Modify: `services/frontend/src/app/page.tsx`
- Modify: `services/frontend/src/app/layout.tsx`

> **Reference:** `docs/UIUX_SPEC.md` section 6.4 — URL params: lat, lng, z, pitch, bearing, layers

**Step 1: Create URL state hook with nuqs**

Create `services/frontend/src/hooks/use-map-url-state.ts`:

```typescript
"use client";

import {
  parseAsFloat,
  parseAsString,
  useQueryStates,
} from "nuqs";
import { useEffect, useRef } from "react";
import { useMapStore } from "@/stores/map-store";
import { MAP_CONFIG } from "@/lib/constants";

const mapParams = {
  lat: parseAsFloat.withDefault(MAP_CONFIG.center[1]),
  lng: parseAsFloat.withDefault(MAP_CONFIG.center[0]),
  z: parseAsFloat.withDefault(MAP_CONFIG.zoom),
  pitch: parseAsFloat.withDefault(MAP_CONFIG.pitch),
  bearing: parseAsFloat.withDefault(MAP_CONFIG.bearing),
  layers: parseAsString.withDefault("landprice,zoning"),
};

export function useMapUrlState() {
  const [params, setParams] = useQueryStates(mapParams, {
    history: "replace",
    shallow: true,
  });
  const initialized = useRef(false);
  const { viewState, setViewState, visibleLayers, toggleLayer } = useMapStore();

  // On mount: restore map state from URL
  useEffect(() => {
    if (initialized.current) return;
    initialized.current = true;

    setViewState({
      latitude: params.lat,
      longitude: params.lng,
      zoom: params.z,
      pitch: params.pitch,
      bearing: params.bearing,
    });

    // Sync layers from URL
    const urlLayers = new Set(params.layers.split(",").filter(Boolean));
    const currentLayers = useMapStore.getState().visibleLayers;
    for (const id of currentLayers) {
      if (!urlLayers.has(id)) toggleLayer(id);
    }
    for (const id of urlLayers) {
      if (!currentLayers.has(id)) toggleLayer(id);
    }
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  // Sync store → URL on view state change
  useEffect(() => {
    if (!initialized.current) return;
    setParams({
      lat: Math.round(viewState.latitude * 10000) / 10000,
      lng: Math.round(viewState.longitude * 10000) / 10000,
      z: Math.round(viewState.zoom * 10) / 10,
      pitch: Math.round(viewState.pitch),
      bearing: Math.round(viewState.bearing),
      layers: [...visibleLayers].sort().join(","),
    });
  }, [viewState, visibleLayers, setParams]);
}
```

**Step 2: Add NuqsAdapter to layout**

Modify `services/frontend/src/app/layout.tsx`: wrap `<Providers>` children with `<NuqsAdapter>` from `nuqs/adapters/next/app`.

**Step 3: Call hook from page.tsx**

Add `useMapUrlState()` call at the top of the Home component in `page.tsx`.

**Step 4: Verify build**

```bash
cd services/frontend && pnpm tsc --noEmit
```

**Step 5: Commit**

```bash
git add services/frontend/
git commit -m "feat(frontend): add URL state sync with nuqs for shareable map views"
```

---

## Task 12: ComparePanel

**Files:**
- Create: `services/frontend/src/components/compare-panel.tsx`
- Modify: `services/frontend/src/app/page.tsx`

> **Reference:** `docs/UIUX_SPEC.md` section 4.5 — ComparePanel with radar chart and detail comparison. FR-5 from REQUIREMENTS.md.

**Step 1: Create ComparePanel with Recharts RadarChart**

Create `services/frontend/src/components/compare-panel.tsx`:

```tsx
"use client";

import { motion, AnimatePresence } from "framer-motion";
import {
  Radar,
  RadarChart,
  PolarGrid,
  PolarAngleAxis,
  ResponsiveContainer,
} from "recharts";
import { useUIStore } from "@/stores/ui-store";
import { useScore } from "@/features/score/api/use-score";

export function ComparePanel() {
  const { compareMode, comparePointA, comparePointB, exitCompareMode } = useUIStore();
  const { data: scoreA } = useScore(comparePointA?.lat ?? null, comparePointA?.lng ?? null);
  const { data: scoreB } = useScore(comparePointB?.lat ?? null, comparePointB?.lng ?? null);

  const showPanel = compareMode && comparePointA !== null && comparePointB !== null;

  const radarData = scoreA && scoreB
    ? [
        { axis: "地価", A: scoreA.components.trend.value, B: scoreB.components.trend.value },
        { axis: "安全性", A: scoreA.components.risk.value, B: scoreB.components.risk.value },
        { axis: "教育", A: Math.min(scoreA.components.access.value, 12.5), B: Math.min(scoreB.components.access.value, 12.5) },
        { axis: "医療", A: Math.max(0, scoreA.components.access.value - 12.5), B: Math.max(0, scoreB.components.access.value - 12.5) },
        { axis: "利回り", A: scoreA.components.yield_potential.value, B: scoreB.components.yield_potential.value },
      ]
    : [];

  return (
    <AnimatePresence>
      {showPanel && (
        <motion.div
          initial={{ scale: 0.9, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          exit={{ scale: 0.9, opacity: 0 }}
          className="fixed inset-0 flex items-center justify-center"
          style={{ zIndex: 100 }}
        >
          {/* Backdrop */}
          <div
            className="absolute inset-0"
            style={{ background: "rgba(0,0,0,0.6)" }}
            onClick={exitCompareMode}
            onKeyDown={(e) => e.key === "Escape" && exitCompareMode()}
            role="button"
            tabIndex={0}
            aria-label="Close comparison"
          />

          {/* Panel */}
          <div
            className="relative rounded-lg p-6 max-w-2xl w-full mx-4"
            style={{
              background: "var(--bg-secondary)",
              border: "1px solid var(--border-primary)",
              backdropFilter: "blur(12px)",
            }}
          >
            <div className="flex justify-between items-center mb-4">
              <span className="text-[9px] tracking-[0.15em]" style={{ color: "var(--accent-cyan)" }}>
                COMPARE ANALYSIS
              </span>
              <button type="button" onClick={exitCompareMode} className="text-sm" style={{ color: "var(--text-muted)" }}>
                ×
              </button>
            </div>

            {/* Point labels */}
            <div className="flex justify-around mb-4">
              <div className="text-center">
                <div className="text-[9px] tracking-[0.15em]" style={{ color: "var(--accent-cyan)" }}>POINT A</div>
                <div className="text-xs" style={{ color: "var(--text-primary)" }}>{comparePointA?.address}</div>
                {scoreA && <div className="text-lg font-bold" style={{ color: "var(--accent-cyan)" }}>{Math.round(scoreA.score)}</div>}
              </div>
              <div className="text-center">
                <div className="text-[9px] tracking-[0.15em]" style={{ color: "var(--accent-warning)" }}>POINT B</div>
                <div className="text-xs" style={{ color: "var(--text-primary)" }}>{comparePointB?.address}</div>
                {scoreB && <div className="text-lg font-bold" style={{ color: "var(--accent-warning)" }}>{Math.round(scoreB.score)}</div>}
              </div>
            </div>

            {/* Radar chart */}
            {radarData.length > 0 && (
              <div style={{ width: "100%", height: 250 }}>
                <ResponsiveContainer>
                  <RadarChart data={radarData}>
                    <PolarGrid stroke="var(--border-primary)" />
                    <PolarAngleAxis
                      dataKey="axis"
                      tick={{ fill: "var(--text-secondary)", fontSize: 10 }}
                    />
                    <Radar name="A" dataKey="A" stroke="var(--accent-cyan)" fill="var(--accent-cyan)" fillOpacity={0.2} />
                    <Radar name="B" dataKey="B" stroke="var(--accent-warning)" fill="var(--accent-warning)" fillOpacity={0.2} />
                  </RadarChart>
                </ResponsiveContainer>
              </div>
            )}
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
```

**Step 2: Add ComparePanel to page.tsx and handle compare mode clicks**

Import and render `<ComparePanel />`. In `handleFeatureClick`, check if `compareMode` is active — if so, call `setComparePoint` instead of `selectFeature`.

**Step 3: Verify build**

```bash
cd services/frontend && pnpm tsc --noEmit
```

**Step 4: Commit**

```bash
git add services/frontend/
git commit -m "feat(frontend): add ComparePanel with radar chart for 2-point comparison"
```

---

## Task 13: Error Boundaries + Loading States

**Files:**
- Create: `services/frontend/src/app/error.tsx`
- Create: `services/frontend/src/app/not-found.tsx`
- Create: `services/frontend/src/components/error-fallback.tsx`

> **Reference:** CLAUDE.md rules — error boundary hierarchy, `error.tsx` must be `'use client'` with retry button.

**Step 1: Create error fallback component**

Create `services/frontend/src/components/error-fallback.tsx`:

```tsx
"use client";

interface ErrorFallbackProps {
  error: Error & { digest?: string };
  reset: () => void;
}

export function ErrorFallback({ error, reset }: ErrorFallbackProps) {
  return (
    <div
      className="flex flex-col items-center justify-center h-screen gap-4"
      style={{
        background: "var(--bg-primary)",
        fontFamily: "var(--font-mono)",
      }}
    >
      <div
        className="text-[9px] tracking-[0.15em]"
        style={{ color: "var(--accent-danger)" }}
      >
        ── SYSTEM ERROR ──
      </div>
      <div className="text-sm" style={{ color: "var(--text-primary)" }}>
        {error.message}
      </div>
      <button
        type="button"
        onClick={reset}
        className="px-4 py-2 rounded text-xs"
        style={{
          background: "var(--bg-tertiary)",
          color: "var(--accent-cyan)",
          border: "1px solid var(--border-primary)",
        }}
      >
        RETRY
      </button>
      <a
        href="/"
        className="text-xs"
        style={{ color: "var(--text-muted)" }}
      >
        RETURN TO BASE
      </a>
    </div>
  );
}
```

**Step 2: Create app-level error boundary**

Create `services/frontend/src/app/error.tsx`:

```tsx
"use client";

import { ErrorFallback } from "@/components/error-fallback";

export default function Error({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  return <ErrorFallback error={error} reset={reset} />;
}
```

**Step 3: Create not-found page**

Create `services/frontend/src/app/not-found.tsx`:

```tsx
export default function NotFound() {
  return (
    <div
      className="flex flex-col items-center justify-center h-screen gap-4"
      style={{
        background: "var(--bg-primary)",
        fontFamily: "var(--font-mono)",
      }}
    >
      <div className="text-[9px] tracking-[0.15em]" style={{ color: "var(--accent-warning)" }}>
        ── 404 — SECTOR NOT FOUND ──
      </div>
      <a href="/" className="text-xs" style={{ color: "var(--accent-cyan)" }}>
        RETURN TO BASE
      </a>
    </div>
  );
}
```

**Step 4: Verify build**

```bash
cd services/frontend && pnpm tsc --noEmit
```

**Step 5: Commit**

```bash
git add services/frontend/
git commit -m "feat(frontend): add error boundaries, not-found, and loading states"
```

---

## Task 14: Responsive Layout + Final Polish

**Files:**
- Modify: `services/frontend/src/app/page.tsx` — responsive breakpoints
- Modify: `services/frontend/src/components/layer-panel.tsx` — collapsible on tablet
- Modify: `services/frontend/src/components/dashboard-stats.tsx` — responsive height

> **Reference:** `docs/UIUX_SPEC.md` section 7 — Responsive breakpoints (desktop 1280+, tablet 768-1279, mobile <768).

**Step 1: Add responsive classes to LayerPanel**

- Desktop (≥1280): Fixed 280px left panel
- Tablet (768-1279): Collapsible with hamburger toggle (use `Sheet` from shadcn)
- Mobile (<768): Bottom sheet

**Step 2: Add responsive classes to ScoreCard**

- Desktop: Fixed 320px right panel
- Tablet: Fixed 280px right panel
- Mobile: Bottom sheet

**Step 3: Add responsive classes to DashboardStats**

- Desktop: Fixed 120px height
- Tablet: Fixed 80px height
- Mobile: Hidden (tap to show)

**Step 4: Final type check + lint + test**

```bash
cd services/frontend && pnpm tsc --noEmit && pnpm biome check . && pnpm vitest run
```

**Step 5: Commit**

```bash
git add services/frontend/
git commit -m "feat(frontend): add responsive layout for desktop/tablet/mobile"
```

---

## Verification Checklist

After all 14 tasks, verify:

- [ ] `pnpm tsc --noEmit` — zero type errors
- [ ] `pnpm biome check .` — zero lint warnings
- [ ] `pnpm vitest run` — all tests pass
- [ ] `pnpm build` — production build succeeds
- [ ] Manual: Map renders with CARTO Dark Matter basemap
- [ ] Manual: LayerPanel toggles layers on/off
- [ ] Manual: Click feature → ScoreCard slides in with score gauge
- [ ] Manual: Price trend sparkline shows CAGR
- [ ] Manual: DashboardStats updates on map pan
- [ ] Manual: URL updates with lat/lng/z/layers params
- [ ] Manual: CRT overlay (vignette + scanlines) visible
- [ ] Manual: Status bar shows coordinates, zoom, demo badge

---

## Feature Requirements Coverage

| Requirement | Task | Component |
|-------------|------|-----------|
| FR-1: Map layers | Task 4, 5, 7 | MapView + 6 layers + data fetching |
| FR-2: Investment scoring | Task 8 | ScoreCard + ScoreGauge |
| FR-3: Score card details | Task 8 | ScoreCard (pricing, risk, facilities) |
| FR-4: Area statistics | Task 10 | DashboardStats |
| FR-5: Comparison | Task 12 | ComparePanel + radar chart |
| FR-6: Price trend | Task 9 | Sparkline |
| FR-7: URL state | Task 11 | nuqs integration |
| FR-8: CRT theme | Existing | CRTOverlay + globals.css |
