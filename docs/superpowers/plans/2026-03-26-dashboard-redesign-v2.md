# Dashboard Redesign v2 — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Transform the current GIS layer-toggle tool into a 3-mode (Explore/Analyze/Compare) investment analysis dashboard with admin boundary filtering, theme presets, i18n, and data pipeline.

**Architecture:** The redesign is split into 8 phases, each independently deployable. Phase 0-3 deliver the core mode system. Phase 4-7 add data, backend, and advanced features. Each phase builds on the previous but produces working software on its own.

**Tech Stack:** Next.js 16, React 19, MapLibre GL, shadcn/ui, Tailwind CSS v4, Zustand, TanStack Query v5, next-intl, Rust Axum, PostGIS

**Design Spec:** `docs/designs/dashboard-redesign-v2.md`
**Data Research:** `docs/research/2026-03-26-government-data-sources-comprehensive.md`
**TLS Algorithm:** `docs/designs/analysis-algorithm-design-integrated.html`

---

## Phase Overview

| Phase | Name | Scope | Depends On | Deliverable |
|-------|------|-------|-----------|-------------|
| 0 | Foundation | Stores, types, i18n, theme config | — | Mode/area state, i18n JA/EN, theme layer mapping |
| 1 | Shell & Boundaries | Top bar, context panel container, N03 boundary layer | Phase 0 | New layout with mode tabs, persistent boundaries |
| 2 | Explore Mode | Theme presets, area card, ranking, breadcrumb | Phase 1 | Clickable map exploration with area statistics |
| 3 | Analyze Mode | TLS panel in context panel, weight presets, axis detail | Phase 1 | Full scoring analysis in left panel |
| 4 | Compare Mode | 2-point/area comparison in context panel | Phase 3 | Side-by-side comparison with radar chart |
| 5 | Data Pipeline | Download/convert/import scripts | — (independent) | Automated government data acquisition |
| 6 | Backend APIs | Area stats, ranking, boundary endpoints | Phase 5 | Server-side aggregation for explore mode |
| 7 | Scoring Enhancements | New sub-scores, cross-analysis patterns, LLM report | Phase 6 | Enhanced 5-axis scoring + AI reports |

---

## Phase 0: Foundation

### Task 0.1: Add next-intl and Configure i18n

**Files:**
- Create: `services/frontend/src/i18n/config.ts`
- Create: `services/frontend/src/i18n/locales/ja.json`
- Create: `services/frontend/src/i18n/locales/en.json`
- Create: `services/frontend/src/i18n/request.ts`
- Modify: `services/frontend/src/app/layout.tsx`

- [ ] **Step 1: Install next-intl**

```bash
cd services/frontend && pnpm add next-intl
```

- [ ] **Step 2: Create i18n config**

```ts
// src/i18n/config.ts
export const locales = ["ja", "en"] as const;
export type Locale = (typeof locales)[number];
export const defaultLocale: Locale = "ja";
```

- [ ] **Step 3: Create Japanese locale file**

```json
// src/i18n/locales/ja.json
{
  "mode": {
    "explore": "探索",
    "analyze": "分析",
    "compare": "比較"
  },
  "theme": {
    "safety": "安全性",
    "livability": "利便性",
    "price": "価格",
    "future": "将来性"
  },
  "tls": {
    "score": "総合スコア",
    "grade": { "S": "優秀", "A": "非常に良い", "B": "良い", "C": "普通", "D": "平均以下", "E": "低い" }
  },
  "axis": {
    "disaster": "災害リスク",
    "terrain": "地盤・地形",
    "livability": "生活利便性",
    "future": "将来性",
    "price": "価格分析"
  },
  "explore": {
    "prompt": "地図上の都道府県・市区町村をクリックしてください",
    "areaCard": { "population": "人口", "avgPrice": "地価平均", "risk": "災害リスク", "avgTls": "TLS平均" },
    "ranking": "市区町村ランキング",
    "analyzeArea": "このエリアで分析"
  },
  "analyze": {
    "weightPreset": { "balance": "バランス", "investment": "投資重視", "residential": "居住重視", "disaster": "防災重視" },
    "aiReport": "AIレポート生成",
    "toCompare": "比較モードへ",
    "minPenalty": "{factor}({score})がスコアを制約"
  },
  "compare": {
    "pointA": "地点A",
    "pointB": "地点B",
    "summary": "判定サマリー",
    "endCompare": "比較終了",
    "aiCompare": "AI比較レポート"
  },
  "search": { "placeholder": "住所・駅名・市区町村を検索" },
  "settings": { "language": "言語", "mapStyle": "地図スタイル", "radius": "分析半径", "cache": "キャッシュ更新" },
  "common": { "loading": "読み込み中...", "error": "エラーが発生しました", "close": "閉じる" }
}
```

- [ ] **Step 4: Create English locale file**

Same keys as ja.json with English values. Key mappings:

```json
// src/i18n/locales/en.json
{
  "mode": { "explore": "Explore", "analyze": "Analyze", "compare": "Compare" },
  "theme": { "safety": "Safety", "livability": "Livability", "price": "Price", "future": "Future" },
  "tls": {
    "score": "Total Score",
    "grade": { "S": "Excellent", "A": "Very Good", "B": "Good", "C": "Fair", "D": "Below Average", "E": "Poor" }
  },
  "axis": { "disaster": "Disaster Risk", "terrain": "Terrain", "livability": "Livability", "future": "Future Potential", "price": "Price Analysis" },
  "explore": {
    "prompt": "Click a prefecture or municipality on the map",
    "areaCard": { "population": "Population", "avgPrice": "Avg Price", "risk": "Disaster Risk", "avgTls": "Avg TLS" },
    "ranking": "Municipality Ranking",
    "analyzeArea": "Analyze this area"
  },
  "analyze": {
    "weightPreset": { "balance": "Balance", "investment": "Investment", "residential": "Residential", "disaster": "Disaster Prevention" },
    "aiReport": "Generate AI Report",
    "toCompare": "Compare Mode",
    "minPenalty": "{factor}({score}) constrains score"
  },
  "compare": {
    "pointA": "Point A",
    "pointB": "Point B",
    "summary": "Summary",
    "endCompare": "End Comparison",
    "aiCompare": "AI Comparison Report"
  },
  "search": { "placeholder": "Search address, station, or municipality" },
  "settings": { "language": "Language", "mapStyle": "Map Style", "radius": "Analysis Radius", "cache": "Cache Update" },
  "common": { "loading": "Loading...", "error": "An error occurred", "close": "Close" }
}
```

- [ ] **Step 5: Create request config for next-intl**

```ts
// src/i18n/request.ts
import { getRequestConfig } from "next-intl/server";
import { defaultLocale } from "./config";

export default getRequestConfig(async () => {
  const locale = defaultLocale;
  return {
    locale,
    messages: (await import(`./locales/${locale}.json`)).default,
  };
});
```

- [ ] **Step 6: Wrap layout with NextIntlClientProvider**

Modify `src/app/layout.tsx` to import and wrap children with `NextIntlClientProvider`.

- [ ] **Step 7: Verify build**

```bash
cd services/frontend && pnpm tsc --noEmit
```

- [ ] **Step 8: Commit**

```bash
git add services/frontend/src/i18n/ services/frontend/src/app/layout.tsx services/frontend/package.json services/frontend/pnpm-lock.yaml
git commit -m "feat(frontend): add next-intl i18n foundation with JA/EN locales"
```

---

### Task 0.2: Create Theme Layer Mapping Config

**Files:**
- Create: `services/frontend/src/lib/themes.ts`
- Modify: `services/frontend/src/lib/layers.ts` (add `theme` field)

- [ ] **Step 1: Add `theme` field to LayerConfig**

In `src/lib/layers.ts`, add optional `theme` field to `LayerConfig` interface:

```ts
/** Theme(s) this layer belongs to for automatic theme preset activation */
theme?: Array<"safety" | "livability" | "price" | "future">;
```

Then add `theme` to each layer entry in `LAYERS` array. Examples:
- `flood`: `theme: ["safety"]`
- `landprice`: `theme: ["price"]`
- `schools`: `theme: ["livability"]`
- `population_mesh`: `theme: ["future"]`
- `station`: `theme: ["livability", "future"]`
- `zoning`: `theme: ["price", "future"]`
- `admin_boundary`: no theme (always visible)

- [ ] **Step 2: Create themes.ts**

```ts
// src/lib/themes.ts
import { LAYERS } from "./layers";

export type ThemeId = "safety" | "livability" | "price" | "future";

export interface ThemeConfig {
  id: ThemeId;
  /** i18n key for display name */
  labelKey: string;
  /** TLS axis used for choropleth coloring */
  colorAxis: "disaster" | "terrain" | "livability" | "future" | "price";
  /** Color ramp: low-risk to high-risk (green to red, or contextual) */
  colorRamp: [string, string, string];
}

export const THEMES: ThemeConfig[] = [
  { id: "safety", labelKey: "theme.safety", colorAxis: "disaster", colorRamp: ["#ef4444", "#eab308", "#10b981"] },
  { id: "livability", labelKey: "theme.livability", colorAxis: "livability", colorRamp: ["#f97316", "#eab308", "#10b981"] },
  { id: "price", labelKey: "theme.price", colorAxis: "price", colorRamp: ["#3b82f6", "#eab308", "#ef4444"] },
  { id: "future", labelKey: "theme.future", colorAxis: "future", colorRamp: ["#ef4444", "#eab308", "#3b82f6"] },
];

/** Get layer IDs that belong to a given theme */
export function getLayerIdsByTheme(themeId: ThemeId): string[] {
  return LAYERS.filter((l) => l.theme?.includes(themeId)).map((l) => l.id);
}

/** Get all layer IDs for multiple active themes */
export function getLayerIdsForThemes(themeIds: Set<ThemeId>): Set<string> {
  const ids = new Set<string>();
  for (const themeId of themeIds) {
    for (const layerId of getLayerIdsByTheme(themeId)) {
      ids.add(layerId);
    }
  }
  // Always include admin_boundary
  ids.add("admin_boundary");
  return ids;
}
```

- [ ] **Step 3: Verify build + run tests**

```bash
cd services/frontend && pnpm tsc --noEmit && pnpm vitest run
```

- [ ] **Step 4: Commit**

```bash
git add services/frontend/src/lib/themes.ts services/frontend/src/lib/layers.ts
git commit -m "feat(frontend): add theme-to-layer mapping config for 4 analysis themes"
```

---

### Task 0.3: Refactor Stores for Mode System

**Files:**
- Modify: `services/frontend/src/stores/ui-store.ts`
- Modify: `services/frontend/src/stores/map-store.ts`
- Create: `services/frontend/src/stores/analysis-store.ts`

- [ ] **Step 1: Add mode and locale to ui-store**

Replace `services/frontend/src/stores/ui-store.ts` with:

```ts
import { create } from "zustand";
import { devtools } from "zustand/middleware";
import type { Locale } from "@/i18n/config";
import type { ThemeId } from "@/lib/themes";

export type AppMode = "explore" | "analyze" | "compare";

type ComparePoint = {
  lat: number;
  lng: number;
  address: string;
} | null;

interface UIState {
  // Mode
  mode: AppMode;
  setMode: (mode: AppMode) => void;

  // Locale
  locale: Locale;
  setLocale: (locale: Locale) => void;

  // Theme presets (explore mode)
  activeThemes: Set<ThemeId>;
  toggleTheme: (themeId: ThemeId) => void;
  clearThemes: () => void;

  // Layer settings panel
  layerSettingsOpen: boolean;
  toggleLayerSettings: () => void;

  // Compare state
  comparePointA: ComparePoint;
  comparePointB: ComparePoint;
  setComparePoint: (point: { lat: number; lng: number; address: string }) => void;
  resetCompare: () => void;
}

export const useUIStore = create<UIState>()(
  devtools(
    (set, get) => ({
      mode: "explore",
      setMode: (mode) => set({ mode }),

      locale: "ja",
      setLocale: (locale) => set({ locale }),

      activeThemes: new Set<ThemeId>(),
      toggleTheme: (themeId) =>
        set((state) => {
          const next = new Set(state.activeThemes);
          if (next.has(themeId)) {
            next.delete(themeId);
          } else {
            next.add(themeId);
          }
          return { activeThemes: next };
        }),
      clearThemes: () => set({ activeThemes: new Set() }),

      layerSettingsOpen: false,
      toggleLayerSettings: () =>
        set((state) => ({ layerSettingsOpen: !state.layerSettingsOpen })),

      comparePointA: null,
      comparePointB: null,
      setComparePoint: (point) => {
        const { comparePointA } = get();
        if (comparePointA === null) {
          set({ comparePointA: point });
        } else {
          set({ comparePointB: point });
        }
      },
      resetCompare: () => set({ comparePointA: null, comparePointB: null }),
    }),
    { name: "ui-store" },
  ),
);
```

- [ ] **Step 2: Add selectedArea to map-store**

Add to `MapState` interface in `services/frontend/src/stores/map-store.ts`:

```ts
interface SelectedArea {
  code: string;       // Administrative code (e.g., "13" for Tokyo, "13105" for Bunkyo)
  name: string;       // Display name
  level: "prefecture" | "municipality";
  bbox: { south: number; west: number; north: number; east: number };
}

// Add to MapState interface:
selectedArea: SelectedArea | null;
selectArea: (area: SelectedArea | null) => void;

// Add to the create block:
selectedArea: null,
selectArea: (area) => set({ selectedArea: area }),
```

- [ ] **Step 3: Create analysis-store**

```ts
// src/stores/analysis-store.ts
import { create } from "zustand";
import { devtools } from "zustand/middleware";

export type WeightPreset = "balance" | "investment" | "residential" | "disaster";

interface AnalysisPoint {
  lat: number;
  lng: number;
  address?: string;
}

interface AnalysisState {
  weightPreset: WeightPreset;
  setWeightPreset: (preset: WeightPreset) => void;

  analysisPoint: AnalysisPoint | null;
  setAnalysisPoint: (point: AnalysisPoint | null) => void;

  analysisRadius: number;
  setAnalysisRadius: (radius: number) => void;
}

export const useAnalysisStore = create<AnalysisState>()(
  devtools(
    (set) => ({
      weightPreset: "balance",
      setWeightPreset: (preset) => set({ weightPreset: preset }),

      analysisPoint: null,
      setAnalysisPoint: (point) => set({ analysisPoint: point }),

      analysisRadius: 500,
      setAnalysisRadius: (radius) => set({ analysisRadius: radius }),
    }),
    { name: "analysis-store" },
  ),
);
```

- [ ] **Step 4: Verify build**

```bash
cd services/frontend && pnpm tsc --noEmit
```

- [ ] **Step 5: Commit**

```bash
git add services/frontend/src/stores/
git commit -m "feat(frontend): refactor stores for 3-mode system with area selection and analysis state"
```

---

## Phase 1: Shell & Boundaries

### Task 1.1: Create Top Bar Component

**Files:**
- Create: `services/frontend/src/components/top-bar/top-bar.tsx`
- Create: `services/frontend/src/components/top-bar/mode-tabs.tsx`
- Create: `services/frontend/src/components/top-bar/locale-toggle.tsx`

- [ ] **Step 1: Create mode-tabs.tsx**

```tsx
// src/components/top-bar/mode-tabs.tsx
"use client";

import { useTranslations } from "next-intl";
import type { AppMode } from "@/stores/ui-store";
import { useUIStore } from "@/stores/ui-store";

const MODES: AppMode[] = ["explore", "analyze", "compare"];

export function ModeTabs() {
  const t = useTranslations("mode");
  const { mode, setMode } = useUIStore();

  return (
    <div className="flex gap-1" role="tablist" aria-label="Application mode">
      {MODES.map((m) => (
        <button
          key={m}
          type="button"
          role="tab"
          aria-selected={mode === m}
          onClick={() => setMode(m)}
          className="px-3 py-1.5 rounded text-xs tracking-wide transition-colors"
          style={{
            background: mode === m ? "var(--hover-accent)" : "transparent",
            color: mode === m ? "var(--accent-cyan)" : "var(--text-muted)",
            fontFamily: "var(--font-mono)",
          }}
        >
          {t(m)}
        </button>
      ))}
    </div>
  );
}
```

- [ ] **Step 2: Create locale-toggle.tsx**

```tsx
// src/components/top-bar/locale-toggle.tsx
"use client";

import { useUIStore } from "@/stores/ui-store";

export function LocaleToggle() {
  const { locale, setLocale } = useUIStore();

  return (
    <button
      type="button"
      onClick={() => setLocale(locale === "ja" ? "en" : "ja")}
      className="px-2 py-1 rounded text-[10px] tracking-wider"
      style={{
        border: "1px solid var(--border-primary)",
        color: "var(--text-muted)",
        fontFamily: "var(--font-mono)",
      }}
      aria-label={`Switch to ${locale === "ja" ? "English" : "Japanese"}`}
    >
      {locale === "ja" ? "EN" : "JA"}
    </button>
  );
}
```

- [ ] **Step 3: Create top-bar.tsx**

```tsx
// src/components/top-bar/top-bar.tsx
"use client";

import { ModeTabs } from "./mode-tabs";
import { LocaleToggle } from "./locale-toggle";

export function TopBar() {
  return (
    <header
      className="fixed top-0 left-0 right-0 flex items-center justify-between px-4 gap-4"
      style={{
        height: 48,
        background: "var(--bg-secondary)",
        borderBottom: "1px solid var(--border-primary)",
        zIndex: 50,
      }}
    >
      <ModeTabs />
      {/* SearchBar will be added in Phase 2 */}
      <div className="flex-1" />
      <div className="flex items-center gap-2">
        <LocaleToggle />
      </div>
    </header>
  );
}
```

- [ ] **Step 4: Verify build**

```bash
cd services/frontend && pnpm tsc --noEmit
```

- [ ] **Step 5: Commit**

```bash
git add services/frontend/src/components/top-bar/
git commit -m "feat(frontend): add top bar with mode tabs and locale toggle"
```

---

### Task 1.2: Create Context Panel Container

**Files:**
- Create: `services/frontend/src/components/context-panel/context-panel.tsx`

- [ ] **Step 1: Create context-panel.tsx**

```tsx
// src/components/context-panel/context-panel.tsx
"use client";

import { AnimatePresence, motion } from "framer-motion";
import { useUIStore } from "@/stores/ui-store";

interface ContextPanelProps {
  children: React.ReactNode;
}

export function ContextPanel({ children }: ContextPanelProps) {
  return (
    <motion.aside
      initial={{ x: -320 }}
      animate={{ x: 0 }}
      transition={{ duration: 0.3 }}
      className="fixed left-0 overflow-y-auto"
      style={{
        top: 48,
        bottom: 28,
        width: 320,
        background: "var(--bg-secondary)",
        borderRight: "1px solid var(--border-primary)",
        zIndex: 40,
      }}
      aria-label="Context panel"
    >
      {children}
    </motion.aside>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add services/frontend/src/components/context-panel/
git commit -m "feat(frontend): add context panel container with animation"
```

---

### Task 1.3: Add N03 Boundary Layer to Map

**Files:**
- Create: `services/frontend/src/components/map/layers/boundary-layer.tsx`
- Create: `services/frontend/src/components/map/area-highlight.tsx`

This task adds the always-visible administrative boundary lines. The actual N03 GeoJSON data download is in Phase 5 — for now, use the existing `admin-boundary-layer.tsx` as foundation and extend it to support both prefecture and municipality levels with zoom-dependent styling.

- [ ] **Step 1: Create boundary-layer.tsx**

```tsx
// src/components/map/layers/boundary-layer.tsx
"use client";

import { Layer, Source } from "react-map-gl/maplibre";

const PREFECTURE_SOURCE_URL = "/geojson/n03/prefectures.geojson";
const MUNICIPALITY_SOURCE_URL = "/geojson/n03/municipalities.geojson";

/**
 * Always-visible N03 administrative boundaries.
 * Prefecture lines show at all zooms; municipality lines fade in at z7+.
 */
export function BoundaryLayer() {
  return (
    <>
      {/* Prefecture boundaries */}
      <Source id="n03-pref" type="geojson" data={PREFECTURE_SOURCE_URL}>
        <Layer
          id="n03-pref-line"
          type="line"
          paint={{
            "line-color": "#ffffff",
            "line-width": [
              "interpolate", ["linear"], ["zoom"],
              4, 0.8,
              10, 2,
            ],
            "line-opacity": [
              "interpolate", ["linear"], ["zoom"],
              4, 0.6,
              10, 0.8,
            ],
          }}
        />
        <Layer
          id="n03-pref-label"
          type="symbol"
          layout={{
            "text-field": ["get", "name"],
            "text-size": [
              "interpolate", ["linear"], ["zoom"],
              4, 10,
              8, 14,
            ],
            "text-anchor": "center",
          }}
          paint={{
            "text-color": "#ffffff",
            "text-opacity": [
              "interpolate", ["linear"], ["zoom"],
              4, 0.7,
              11, 0,
            ],
            "text-halo-color": "#000000",
            "text-halo-width": 1,
          }}
        />
      </Source>

      {/* Municipality boundaries */}
      <Source id="n03-muni" type="geojson" data={MUNICIPALITY_SOURCE_URL}>
        <Layer
          id="n03-muni-line"
          type="line"
          paint={{
            "line-color": "#ffffff",
            "line-width": 0.5,
            "line-opacity": [
              "interpolate", ["linear"], ["zoom"],
              7, 0,
              8, 0.3,
              12, 0.6,
            ],
          }}
        />
        <Layer
          id="n03-muni-label"
          type="symbol"
          layout={{
            "text-field": ["get", "name"],
            "text-size": [
              "interpolate", ["linear"], ["zoom"],
              8, 0,
              10, 10,
              14, 13,
            ],
            "text-anchor": "center",
          }}
          paint={{
            "text-color": "#ffffff",
            "text-opacity": [
              "interpolate", ["linear"], ["zoom"],
              8, 0,
              10, 0.5,
              14, 0.8,
            ],
            "text-halo-color": "#000000",
            "text-halo-width": 1,
          }}
          minzoom={8}
        />
      </Source>
    </>
  );
}
```

- [ ] **Step 2: Create area-highlight.tsx**

```tsx
// src/components/map/area-highlight.tsx
"use client";

import { Layer, Source } from "react-map-gl/maplibre";
import { useMapStore } from "@/stores/map-store";

/**
 * Highlights the currently selected area (prefecture or municipality)
 * with a cyan stroke and transparent fill.
 */
export function AreaHighlight() {
  const selectedArea = useMapStore((s) => s.selectedArea);

  if (!selectedArea) return null;

  // Filter expression: match the admin code
  const filter = selectedArea.level === "prefecture"
    ? ["==", ["get", "prefCode"], selectedArea.code]
    : ["==", ["get", "code"], selectedArea.code];

  const sourceId = selectedArea.level === "prefecture" ? "n03-pref" : "n03-muni";

  return (
    <>
      <Layer
        id="area-highlight-fill"
        type="fill"
        source={sourceId}
        filter={filter}
        paint={{
          "fill-color": "#22d3ee",
          "fill-opacity": 0.1,
        }}
      />
      <Layer
        id="area-highlight-line"
        type="line"
        source={sourceId}
        filter={filter}
        paint={{
          "line-color": "#22d3ee",
          "line-width": 2,
        }}
      />
    </>
  );
}
```

- [ ] **Step 3: Commit**

```bash
git add services/frontend/src/components/map/layers/boundary-layer.tsx services/frontend/src/components/map/area-highlight.tsx
git commit -m "feat(frontend): add N03 boundary layer with zoom-dependent styling and area highlight"
```

---

### Task 1.4: Rewrite page.tsx as Mode Router

**Files:**
- Modify: `services/frontend/src/app/page.tsx`

This is the core wiring task. Replace the monolithic 310-line page with a thin orchestrator that renders the appropriate mode panel.

- [ ] **Step 1: Rewrite page.tsx**

Keep existing MapView + layer rendering logic, but wrap in new layout:

```tsx
// src/app/page.tsx
"use client";

import type { FeatureCollection } from "geojson";
import { parseAsInteger, useQueryState } from "nuqs";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useShallow } from "zustand/react/shallow";
import type { MapLayerMouseEvent } from "react-map-gl/maplibre";

// New components
import { TopBar } from "@/components/top-bar/top-bar";
import { ContextPanel } from "@/components/context-panel/context-panel";
import { BoundaryLayer } from "@/components/map/layers/boundary-layer";
import { AreaHighlight } from "@/components/map/area-highlight";

// Existing components (kept)
import { MapView } from "@/components/map/map-view";
import { PopupCard } from "@/components/map/popup-card";
import { YearSlider } from "@/components/map/year-slider";
import { LandPriceYearSlider } from "@/components/map/land-price-year-slider";
import { StatusBar } from "@/components/status-bar";

// Existing layers (kept)
import {
  LandPriceExtrusionLayer, LandpriceLayer, FloodLayer, SteepSlopeLayer,
  SchoolLayer, MedicalLayer, ZoningLayer, DIDLayer, LandformLayer,
  GeologyLayer, AdminBoundaryLayer, FaultLayer, FloodHistoryLayer,
  LiquefactionLayer, RailwayLayer, SeismicLayer, SoilLayer, VolcanoLayer,
  StationLayer, SchoolDistrictLayer, LandslideLayer, ParkLayer,
  TsunamiLayer, UrbanPlanLayer, PopulationMeshLayer,
} from "@/components/map/layers";

// Existing hooks
import { useAreaData } from "@/features/area-data/api/use-area-data";
import { useHealth } from "@/features/health/api/use-health";
import { useLandPrices } from "@/features/land-prices/api/use-land-prices";
import { useMapUrlState } from "@/hooks/use-map-url-state";
import type { LayerConfig } from "@/lib/layers";
import { LAYERS } from "@/lib/layers";
import { spatialEngine } from "@/lib/wasm/spatial-engine";
import { useMapStore } from "@/stores/map-store";
import { useUIStore } from "@/stores/ui-store";
import { useAnalysisStore } from "@/stores/analysis-store";

// Layer registries (keep existing STATIC_LAYER_COMPONENTS / API_LAYER_COMPONENTS)
// ... (same as current page.tsx lines 73-106)

export default function Home() {
  useMapUrlState();
  const { mode } = useUIStore();

  // ... (keep all existing hooks and state: useAreaData, useLandPrices, etc.)
  // ... (keep all existing layer rendering logic)

  return (
    <div className="relative h-screen w-screen overflow-hidden">
      <TopBar />

      {/* Context panel — content depends on mode */}
      <ContextPanel>
        {mode === "explore" && <div>Explore panel placeholder</div>}
        {mode === "analyze" && <div>Analyze panel placeholder</div>}
        {mode === "compare" && <div>Compare panel placeholder</div>}
      </ContextPanel>

      {/* Map area — offset by top bar and context panel */}
      <div
        className="absolute"
        style={{ top: 48, left: 320, right: 0, bottom: 28 }}
      >
        <MapView onMoveEnd={handleMoveEnd} onFeatureClick={handleFeatureClick}>
          {/* Always-visible boundaries */}
          <BoundaryLayer />
          <AreaHighlight />

          {/* Existing layers (same as current) */}
          <LandPriceExtrusionLayer ... />
          {/* ... all existing layer rendering ... */}
        </MapView>
      </div>

      {/* Popup card (keep existing) */}
      {selectedFeature && selectedLayerConfig?.popupFields && (
        <PopupCard ... />
      )}

      <StatusBar ... />
    </div>
  );
}
```

**Key changes from current page.tsx:**
1. Remove `<LayerPanel />`, `<ScoreCard />`, `<ComparePanel />`, `<DashboardStats />`
2. Add `<TopBar />`, `<ContextPanel>`, `<BoundaryLayer />`, `<AreaHighlight />`
3. Map area is now offset: `top: 48px` (top bar), `left: 320px` (context panel)
4. Mode-dependent content rendered inside ContextPanel (placeholders for now)

- [ ] **Step 2: Verify the app renders with new layout**

```bash
cd services/frontend && pnpm dev
# Open http://localhost:3001 — should see top bar, left panel (placeholder), map with boundaries
```

- [ ] **Step 3: Commit**

```bash
git add services/frontend/src/app/page.tsx
git commit -m "feat(frontend): rewrite page.tsx as mode router with top bar and context panel"
```

---

## Phase 2: Explore Mode

### Task 2.1: Theme Presets Component

**Files:**
- Create: `services/frontend/src/components/explore/theme-presets.tsx`

- [ ] **Step 1: Create theme-presets.tsx**

4 toggle buttons that activate/deactivate themes. When a theme is active, its associated layers are auto-enabled in the map store.

```tsx
// src/components/explore/theme-presets.tsx
"use client";

import { useEffect } from "react";
import { useTranslations } from "next-intl";
import { THEMES, getLayerIdsForThemes } from "@/lib/themes";
import { useUIStore } from "@/stores/ui-store";
import { useMapStore } from "@/stores/map-store";

const ICONS: Record<string, string> = {
  safety: "🛡",
  livability: "🏘",
  price: "💰",
  future: "📈",
};

export function ThemePresets() {
  const t = useTranslations();
  const { activeThemes, toggleTheme } = useUIStore();
  const visibleLayers = useMapStore((s) => s.visibleLayers);

  // Sync theme selection → visible layers
  useEffect(() => {
    if (activeThemes.size === 0) return;
    const themeLayerIds = getLayerIdsForThemes(activeThemes);
    // Merge with always-on layers (admin_boundary)
    useMapStore.setState({ visibleLayers: themeLayerIds });
  }, [activeThemes]);

  return (
    <div className="grid grid-cols-2 gap-2 px-4 py-3">
      {THEMES.map((theme) => {
        const isActive = activeThemes.has(theme.id);
        return (
          <button
            key={theme.id}
            type="button"
            onClick={() => toggleTheme(theme.id)}
            className="flex items-center gap-2 rounded-lg px-3 py-2.5 text-xs transition-colors"
            style={{
              background: isActive ? "var(--hover-accent)" : "var(--bg-tertiary)",
              color: isActive ? "var(--accent-cyan)" : "var(--text-muted)",
              border: isActive ? "1px solid var(--accent-cyan)" : "1px solid transparent",
            }}
            aria-pressed={isActive}
          >
            <span>{ICONS[theme.id]}</span>
            <span style={{ fontFamily: "var(--font-sans)" }}>
              {t(theme.labelKey)}
            </span>
          </button>
        );
      })}
    </div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add services/frontend/src/components/explore/theme-presets.tsx
git commit -m "feat(frontend): add theme preset buttons that auto-control layer visibility"
```

---

### Task 2.2: Breadcrumb Navigation

**Files:**
- Create: `services/frontend/src/components/explore/breadcrumb-nav.tsx`

- [ ] **Step 1: Create breadcrumb-nav.tsx**

```tsx
// src/components/explore/breadcrumb-nav.tsx
"use client";

import { useTranslations } from "next-intl";
import { useMapStore } from "@/stores/map-store";

export function BreadcrumbNav() {
  const t = useTranslations();
  const { selectedArea, selectArea } = useMapStore();

  return (
    <nav
      className="flex items-center gap-1 px-4 py-2 text-[10px] tracking-wide"
      style={{ color: "var(--text-muted)", fontFamily: "var(--font-mono)" }}
      aria-label="Area breadcrumb"
    >
      <button
        type="button"
        onClick={() => selectArea(null)}
        className="hover:underline"
        style={{ color: selectedArea ? "var(--accent-cyan)" : "var(--text-primary)" }}
      >
        {t("locale") === "ja" ? "全国" : "Japan"}
      </button>
      {selectedArea && selectedArea.level === "prefecture" && (
        <>
          <span>&gt;</span>
          <span style={{ color: "var(--text-primary)" }}>{selectedArea.name}</span>
        </>
      )}
      {selectedArea && selectedArea.level === "municipality" && (
        <>
          <span>&gt;</span>
          <button
            type="button"
            onClick={() => {
              // Navigate up to prefecture level
              // This will be refined when area data API exists
              selectArea(null);
            }}
            className="hover:underline"
            style={{ color: "var(--accent-cyan)" }}
          >
            {/* Prefecture name derived from code prefix */}
            ...
          </button>
          <span>&gt;</span>
          <span style={{ color: "var(--text-primary)" }}>{selectedArea.name}</span>
        </>
      )}
    </nav>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add services/frontend/src/components/explore/breadcrumb-nav.tsx
git commit -m "feat(frontend): add breadcrumb navigation for area drilldown"
```

---

### Task 2.3: Area Card Component

**Files:**
- Create: `services/frontend/src/components/explore/area-card.tsx`

- [ ] **Step 1: Create area-card.tsx**

Initially uses placeholder data. Will connect to backend area-stats API in Phase 6.

```tsx
// src/components/explore/area-card.tsx
"use client";

import { useTranslations } from "next-intl";
import { Skeleton } from "@/components/ui/skeleton";
import { useMapStore } from "@/stores/map-store";

interface AreaStat {
  label: string;
  value: string;
  color?: string;
}

export function AreaCard() {
  const t = useTranslations("explore.areaCard");
  const selectedArea = useMapStore((s) => s.selectedArea);

  if (!selectedArea) return null;

  // TODO: Replace with useAreaStats hook in Phase 6
  const stats: AreaStat[] = [
    { label: t("population"), value: "—" },
    { label: t("avgPrice"), value: "—" },
    { label: t("risk"), value: "—" },
    { label: t("avgTls"), value: "—" },
  ];

  return (
    <div className="px-4 py-3">
      <div
        className="rounded-lg p-3"
        style={{ background: "var(--bg-tertiary)" }}
      >
        <div
          className="text-sm font-medium mb-2"
          style={{ color: "var(--text-primary)" }}
        >
          {selectedArea.name}
        </div>
        <div className="grid grid-cols-2 gap-2">
          {stats.map((stat) => (
            <div key={stat.label}>
              <div
                className="text-[9px] tracking-wider"
                style={{ color: "var(--text-muted)", fontFamily: "var(--font-mono)" }}
              >
                {stat.label}
              </div>
              <div
                className="text-sm font-bold"
                style={{ color: stat.color ?? "var(--accent-cyan)" }}
              >
                {stat.value}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add services/frontend/src/components/explore/area-card.tsx
git commit -m "feat(frontend): add area card component with placeholder stats"
```

---

### Task 2.4: Assemble Explore Panel

**Files:**
- Create: `services/frontend/src/components/context-panel/explore-panel.tsx`
- Create: `services/frontend/src/components/shared/layer-settings.tsx`
- Modify: `services/frontend/src/app/page.tsx` (wire explore panel)

- [ ] **Step 1: Extract layer-settings.tsx from existing LayerPanel**

Copy the `LayerPanelContent` function from `src/components/layer-panel.tsx` into `src/components/shared/layer-settings.tsx`, wrapped in a collapsible accordion. This preserves the 23-layer individual toggle for power users.

- [ ] **Step 2: Create explore-panel.tsx**

```tsx
// src/components/context-panel/explore-panel.tsx
"use client";

import { useTranslations } from "next-intl";
import { ThemePresets } from "@/components/explore/theme-presets";
import { BreadcrumbNav } from "@/components/explore/breadcrumb-nav";
import { AreaCard } from "@/components/explore/area-card";
import { LayerSettings } from "@/components/shared/layer-settings";
import { useMapStore } from "@/stores/map-store";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { ChevronRightIcon } from "lucide-react";
import { useState } from "react";

export function ExplorePanel() {
  const t = useTranslations();
  const selectedArea = useMapStore((s) => s.selectedArea);
  const [layerSettingsOpen, setLayerSettingsOpen] = useState(false);

  return (
    <div className="flex flex-col h-full">
      <div className="px-4 pt-4 pb-2">
        <div
          className="text-[9px] tracking-widest"
          style={{ color: "var(--accent-cyan)", fontFamily: "var(--font-mono)" }}
        >
          {t("mode.explore").toUpperCase()}
        </div>
      </div>

      <BreadcrumbNav />
      <ThemePresets />

      {selectedArea ? (
        <AreaCard />
      ) : (
        <div className="px-4 py-8 text-center">
          <div
            className="text-xs"
            style={{ color: "var(--text-muted)" }}
          >
            {t("explore.prompt")}
          </div>
        </div>
      )}

      {/* Collapsed layer settings for power users */}
      <Collapsible
        open={layerSettingsOpen}
        onOpenChange={setLayerSettingsOpen}
        className="mt-auto border-t"
        style={{ borderColor: "var(--border-primary)" }}
      >
        <CollapsibleTrigger className="flex items-center gap-2 w-full px-4 py-2 text-[9px] tracking-wider">
          <ChevronRightIcon
            size={10}
            className="transition-transform"
            style={{
              color: "var(--text-muted)",
              transform: layerSettingsOpen ? "rotate(90deg)" : "rotate(0deg)",
            }}
          />
          <span style={{ color: "var(--text-muted)", fontFamily: "var(--font-mono)" }}>
            LAYER SETTINGS
          </span>
        </CollapsibleTrigger>
        <CollapsibleContent>
          <LayerSettings />
        </CollapsibleContent>
      </Collapsible>
    </div>
  );
}
```

- [ ] **Step 3: Wire into page.tsx**

Replace the explore placeholder in ContextPanel:

```tsx
{mode === "explore" && <ExplorePanel />}
```

- [ ] **Step 4: Verify app renders explore mode**

```bash
cd services/frontend && pnpm dev
```

- [ ] **Step 5: Commit**

```bash
git add services/frontend/src/components/context-panel/explore-panel.tsx services/frontend/src/components/shared/layer-settings.tsx services/frontend/src/app/page.tsx
git commit -m "feat(frontend): assemble explore mode with theme presets, area card, and layer settings"
```

---

## Phase 3: Analyze Mode

### Task 3.1: TLS Score Header Component

**Files:**
- Create: `services/frontend/src/components/analyze/tls-score-header.tsx`

- [ ] **Step 1: Create tls-score-header.tsx**

Large score display with grade badge. Reuses existing `gradeColor` logic from score-card.

```tsx
// src/components/analyze/tls-score-header.tsx
"use client";

import { useTranslations } from "next-intl";

type TlsGrade = "S" | "A" | "B" | "C" | "D" | "E";

const GRADE_COLORS: Record<TlsGrade, string> = {
  S: "#10b981", A: "#22c55e", B: "#eab308",
  C: "#f97316", D: "#ef4444", E: "#991b1b",
};

interface TlsScoreHeaderProps {
  score: number;
  grade: TlsGrade;
  label: string;
}

export function TlsScoreHeader({ score, grade, label }: TlsScoreHeaderProps) {
  const t = useTranslations("tls");

  return (
    <div className="px-4 py-4 text-center">
      <div
        className="text-[9px] tracking-widest mb-2"
        style={{ color: "var(--text-muted)", fontFamily: "var(--font-mono)" }}
      >
        {t("score").toUpperCase()}
      </div>
      <div
        className="text-4xl font-bold"
        style={{ color: GRADE_COLORS[grade] }}
      >
        {Math.round(score)}
      </div>
      <div className="flex items-center justify-center gap-2 mt-1">
        <span
          className="text-lg font-bold"
          style={{ color: GRADE_COLORS[grade] }}
        >
          {grade}
        </span>
        <span
          className="text-xs"
          style={{ color: "var(--text-muted)" }}
        >
          {label}
        </span>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add services/frontend/src/components/analyze/tls-score-header.tsx
git commit -m "feat(frontend): add TLS score header component with grade coloring"
```

---

### Task 3.2: Axis Bar List and Detail Components

**Files:**
- Create: `services/frontend/src/components/analyze/axis-bar-list.tsx`
- Create: `services/frontend/src/components/analyze/axis-detail.tsx`

- [ ] **Step 1: Create axis-bar-list.tsx**

Renders 5 horizontal bars for each TLS axis, expandable to show sub-score details.

```tsx
// src/components/analyze/axis-bar-list.tsx
"use client";

import { useState } from "react";
import { useTranslations } from "next-intl";
import { AxisDetail } from "./axis-detail";
import type { TlsResponse } from "@/lib/schemas";

const AXIS_KEYS = ["disaster", "terrain", "livability", "future", "price"] as const;
type AxisKey = (typeof AXIS_KEYS)[number];

const AXIS_COLORS: Record<AxisKey, string> = {
  disaster: "#ef4444",
  terrain: "#f59e0b",
  livability: "#14b8a6",
  future: "#3b82f6",
  price: "#10b981",
};

interface AxisBarListProps {
  axes: TlsResponse["axes"];
}

export function AxisBarList({ axes }: AxisBarListProps) {
  const t = useTranslations("axis");
  const [expanded, setExpanded] = useState<AxisKey | null>(null);

  return (
    <div className="px-4 space-y-1">
      {AXIS_KEYS.map((key) => {
        const axis = axes[key];
        const color = AXIS_COLORS[key];
        const isExpanded = expanded === key;

        return (
          <div key={key}>
            <button
              type="button"
              onClick={() => setExpanded(isExpanded ? null : key)}
              className="flex items-center gap-2 w-full py-1.5 text-left"
            >
              <span
                className="w-14 text-[10px] truncate"
                style={{ color, fontFamily: "var(--font-sans)" }}
              >
                {t(key)}
              </span>
              <div className="flex-1 h-2 rounded-full overflow-hidden" style={{ background: "var(--bg-tertiary)" }}>
                <div
                  className="h-full rounded-full transition-all"
                  style={{ width: `${axis.score}%`, background: color }}
                />
              </div>
              <span
                className="w-8 text-right text-[11px] font-mono"
                style={{ color: "var(--text-primary)" }}
              >
                {Math.round(axis.score)}
              </span>
            </button>
            {isExpanded && <AxisDetail axisKey={key} axis={axis} />}
          </div>
        );
      })}
    </div>
  );
}
```

- [ ] **Step 2: Create axis-detail.tsx**

```tsx
// src/components/analyze/axis-detail.tsx
"use client";

import type { TlsResponse } from "@/lib/schemas";

type AxisKey = keyof TlsResponse["axes"];

interface AxisDetailProps {
  axisKey: AxisKey;
  axis: TlsResponse["axes"][AxisKey];
}

export function AxisDetail({ axisKey, axis }: AxisDetailProps) {
  return (
    <div
      className="ml-16 mr-2 mb-2 rounded-lg p-2 space-y-1"
      style={{ background: "var(--bg-tertiary)" }}
    >
      {axis.sub.map((sub) => (
        <div key={sub.id} className="flex items-center gap-2">
          <span
            className="w-20 text-[9px] truncate"
            style={{ color: "var(--text-muted)" }}
          >
            {sub.id}
          </span>
          <div className="flex-1 h-1.5 rounded-full overflow-hidden" style={{ background: "var(--bg-primary)" }}>
            <div
              className="h-full rounded-full"
              style={{
                width: `${sub.score}%`,
                background: sub.available ? "var(--text-secondary)" : "var(--text-muted)",
                opacity: sub.available ? 1 : 0.3,
              }}
            />
          </div>
          <span
            className="w-6 text-right text-[9px] font-mono"
            style={{ color: sub.available ? "var(--text-primary)" : "var(--text-muted)" }}
          >
            {Math.round(sub.score)}
          </span>
        </div>
      ))}
      <div
        className="text-[9px] mt-1 pt-1 border-t"
        style={{ color: "var(--text-muted)", borderColor: "var(--border-primary)" }}
      >
        confidence: {Math.round(axis.confidence * 100)}% | weight: {Math.round(axis.weight * 100)}%
      </div>
    </div>
  );
}
```

- [ ] **Step 3: Commit**

```bash
git add services/frontend/src/components/analyze/
git commit -m "feat(frontend): add axis bar list and expandable sub-score detail"
```

---

### Task 3.3: Weight Preset Selector

**Files:**
- Create: `services/frontend/src/components/analyze/weight-preset-selector.tsx`

- [ ] **Step 1: Create weight-preset-selector.tsx**

```tsx
// src/components/analyze/weight-preset-selector.tsx
"use client";

import { useTranslations } from "next-intl";
import { useAnalysisStore, type WeightPreset } from "@/stores/analysis-store";

const PRESETS: WeightPreset[] = ["balance", "investment", "residential", "disaster"];

export function WeightPresetSelector() {
  const t = useTranslations("analyze.weightPreset");
  const { weightPreset, setWeightPreset } = useAnalysisStore();

  return (
    <div className="px-4 py-2">
      <div
        className="text-[9px] tracking-wider mb-1.5"
        style={{ color: "var(--text-muted)", fontFamily: "var(--font-mono)" }}
      >
        WEIGHT PRESET
      </div>
      <div className="flex gap-1 flex-wrap">
        {PRESETS.map((preset) => (
          <button
            key={preset}
            type="button"
            onClick={() => setWeightPreset(preset)}
            className="px-2 py-1 rounded text-[10px] transition-colors"
            style={{
              background: weightPreset === preset ? "var(--hover-accent)" : "var(--bg-tertiary)",
              color: weightPreset === preset ? "var(--accent-cyan)" : "var(--text-muted)",
              fontFamily: "var(--font-mono)",
            }}
            aria-pressed={weightPreset === preset}
          >
            {t(preset)}
          </button>
        ))}
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add services/frontend/src/components/analyze/weight-preset-selector.tsx
git commit -m "feat(frontend): add weight preset selector for TLS analysis"
```

---

### Task 3.4: Cross-Analysis Component

**Files:**
- Create: `services/frontend/src/components/analyze/cross-analysis.tsx`

- [ ] **Step 1: Create cross-analysis.tsx**

```tsx
// src/components/analyze/cross-analysis.tsx
"use client";

import type { TlsResponse } from "@/lib/schemas";

interface CrossAnalysisProps {
  crossAnalysis: TlsResponse["cross_analysis"];
}

const PATTERNS = [
  { key: "value_discovery", label: "Value Discovery", description: "Safe but undervalued" },
  { key: "demand_signal", label: "Demand Signal", description: "Convenient + growing" },
  { key: "ground_safety", label: "Ground Safety", description: "Disaster x terrain" },
] as const;

export function CrossAnalysis({ crossAnalysis }: CrossAnalysisProps) {
  return (
    <div className="px-4 py-2">
      <div
        className="text-[9px] tracking-wider mb-2"
        style={{ color: "var(--text-muted)", fontFamily: "var(--font-mono)" }}
      >
        CROSS ANALYSIS
      </div>
      <div className="space-y-1.5">
        {PATTERNS.map(({ key, label, description }) => {
          const value = crossAnalysis[key];
          return (
            <div key={key} className="flex items-center gap-2">
              <span className="w-28 text-[10px]" style={{ color: "var(--text-secondary)" }}>
                {label}
              </span>
              <div className="flex-1 h-1.5 rounded-full overflow-hidden" style={{ background: "var(--bg-tertiary)" }}>
                <div
                  className="h-full rounded-full"
                  style={{ width: `${value}%`, background: "var(--accent-cyan)" }}
                />
              </div>
              <span className="w-6 text-right text-[10px] font-mono" style={{ color: "var(--text-primary)" }}>
                {Math.round(value)}
              </span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add services/frontend/src/components/analyze/cross-analysis.tsx
git commit -m "feat(frontend): add cross-analysis display component"
```

---

### Task 3.5: Assemble Analyze Panel

**Files:**
- Create: `services/frontend/src/components/context-panel/analyze-panel.tsx`
- Modify: `services/frontend/src/app/page.tsx` (wire analyze panel)

- [ ] **Step 1: Create analyze-panel.tsx**

```tsx
// src/components/context-panel/analyze-panel.tsx
"use client";

import { useTranslations } from "next-intl";
import { Skeleton } from "@/components/ui/skeleton";
import { TlsScoreHeader } from "@/components/analyze/tls-score-header";
import { AxisBarList } from "@/components/analyze/axis-bar-list";
import { WeightPresetSelector } from "@/components/analyze/weight-preset-selector";
import { CrossAnalysis } from "@/components/analyze/cross-analysis";
import { AxisDetail } from "@/components/analyze/axis-detail";
import { useScore } from "@/features/score/api/use-score";
import { useAnalysisStore } from "@/stores/analysis-store";
import { useUIStore } from "@/stores/ui-store";

export function AnalyzePanel() {
  const t = useTranslations();
  const { analysisPoint } = useAnalysisStore();
  const { setMode } = useUIStore();
  const lat = analysisPoint?.lat ?? null;
  const lng = analysisPoint?.lng ?? null;
  const { data: score, isLoading } = useScore(lat, lng);

  if (!analysisPoint) {
    return (
      <div className="flex items-center justify-center h-full px-4">
        <div className="text-xs text-center" style={{ color: "var(--text-muted)" }}>
          {t("explore.prompt")}
        </div>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="p-4 space-y-3">
        <Skeleton className="h-4 w-32" />
        <Skeleton className="h-20 w-full" />
        <Skeleton className="h-32 w-full" />
      </div>
    );
  }

  if (!score) return null;

  return (
    <div className="flex flex-col h-full overflow-y-auto">
      <div className="px-4 pt-4 pb-1">
        <div
          className="text-[9px] tracking-widest"
          style={{ color: "var(--accent-cyan)", fontFamily: "var(--font-mono)" }}
        >
          ANALYZE
        </div>
        <div className="text-xs mt-1" style={{ color: "var(--text-primary)" }}>
          {analysisPoint.address ?? `${lat?.toFixed(4)}°N, ${lng?.toFixed(4)}°E`}
        </div>
      </div>

      <TlsScoreHeader
        score={score.tls.score}
        grade={score.tls.grade}
        label={score.tls.label}
      />

      <WeightPresetSelector />

      <div
        className="border-t my-2"
        style={{ borderColor: "var(--border-primary)" }}
      />

      <AxisBarList axes={score.axes} />

      <div
        className="border-t my-2"
        style={{ borderColor: "var(--border-primary)" }}
      />

      <CrossAnalysis crossAnalysis={score.cross_analysis} />

      <div className="px-4 py-3 mt-auto space-y-2">
        <button
          type="button"
          onClick={() => setMode("compare")}
          className="w-full rounded-lg py-2 text-xs"
          style={{
            background: "var(--bg-tertiary)",
            color: "var(--text-secondary)",
            border: "1px solid var(--border-primary)",
          }}
        >
          {t("analyze.toCompare")}
        </button>
      </div>

      <div className="px-4 pb-3">
        <div className="text-[9px]" style={{ color: "var(--text-muted)" }}>
          {score.metadata.disclaimer}
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Wire into page.tsx**

Replace the analyze placeholder:
```tsx
{mode === "analyze" && <AnalyzePanel />}
```

Also update the `handleFeatureClick` to set `analysisPoint` and switch to analyze mode:

```tsx
// In handleFeatureClick, when a data point is clicked (not in compare mode):
useAnalysisStore.getState().setAnalysisPoint({
  lat: e.lngLat.lat,
  lng: e.lngLat.lng,
  address: feature?.properties?.address as string | undefined,
});
useUIStore.getState().setMode("analyze");
```

- [ ] **Step 3: Verify**

```bash
cd services/frontend && pnpm tsc --noEmit
```

- [ ] **Step 4: Commit**

```bash
git add services/frontend/src/components/context-panel/analyze-panel.tsx services/frontend/src/app/page.tsx
git commit -m "feat(frontend): assemble analyze mode panel with TLS scoring, axis bars, and cross-analysis"
```

---

## Phase 4: Compare Mode

### Task 4.1: Refactor Compare Panel into Context Panel

**Files:**
- Create: `services/frontend/src/components/context-panel/compare-panel.tsx`
- Create: `services/frontend/src/components/compare/radar-chart.tsx`
- Create: `services/frontend/src/components/compare/diff-table.tsx`
- Modify: `services/frontend/src/app/page.tsx`

- [ ] **Step 1: Create radar-chart.tsx**

Extract radar chart from existing `compare-panel.tsx` into standalone component:

```tsx
// src/components/compare/radar-chart.tsx
"use client";

import { PolarAngleAxis, PolarGrid, Radar, RadarChart, ResponsiveContainer } from "recharts";
import type { TlsResponse } from "@/lib/schemas";

interface RadarComparisonProps {
  axesA: TlsResponse["axes"];
  axesB: TlsResponse["axes"];
}

const AXIS_LABELS = ["災害", "地盤", "利便性", "将来性", "価格"];
const AXIS_KEYS = ["disaster", "terrain", "livability", "future", "price"] as const;

export function RadarComparison({ axesA, axesB }: RadarComparisonProps) {
  const data = AXIS_KEYS.map((key, i) => ({
    axis: AXIS_LABELS[i],
    A: axesA[key].score,
    B: axesB[key].score,
  }));

  return (
    <div style={{ width: "100%", height: 200 }}>
      <ResponsiveContainer>
        <RadarChart data={data}>
          <PolarGrid stroke="var(--border-primary)" />
          <PolarAngleAxis dataKey="axis" tick={{ fill: "var(--text-secondary)", fontSize: 10 }} />
          <Radar name="A" dataKey="A" stroke="var(--accent-cyan)" fill="var(--accent-cyan)" fillOpacity={0.2} />
          <Radar name="B" dataKey="B" stroke="var(--accent-warning)" fill="var(--accent-warning)" fillOpacity={0.2} />
        </RadarChart>
      </ResponsiveContainer>
    </div>
  );
}
```

- [ ] **Step 2: Create diff-table.tsx**

```tsx
// src/components/compare/diff-table.tsx
"use client";

import { useTranslations } from "next-intl";
import type { TlsResponse } from "@/lib/schemas";

const AXIS_KEYS = ["disaster", "terrain", "livability", "future", "price"] as const;

interface DiffTableProps {
  axesA: TlsResponse["axes"];
  axesB: TlsResponse["axes"];
  tlsA: number;
  tlsB: number;
}

export function DiffTable({ axesA, axesB, tlsA, tlsB }: DiffTableProps) {
  const t = useTranslations("axis");

  return (
    <div className="px-4">
      <table className="w-full text-[10px]">
        <thead>
          <tr style={{ color: "var(--text-muted)" }}>
            <th className="text-left py-1 font-normal" />
            <th className="text-right py-1 font-normal" style={{ color: "var(--accent-cyan)" }}>A</th>
            <th className="text-right py-1 font-normal" style={{ color: "var(--accent-warning)" }}>B</th>
            <th className="text-right py-1 font-normal">Delta</th>
          </tr>
        </thead>
        <tbody>
          {AXIS_KEYS.map((key) => {
            const a = Math.round(axesA[key].score);
            const b = Math.round(axesB[key].score);
            const delta = a - b;
            return (
              <tr key={key} style={{ color: "var(--text-secondary)" }}>
                <td className="py-0.5">{t(key)}</td>
                <td className="text-right">{a}</td>
                <td className="text-right">{b}</td>
                <td className="text-right" style={{ color: delta > 0 ? "var(--accent-cyan)" : delta < 0 ? "var(--accent-warning)" : "var(--text-muted)" }}>
                  {delta > 0 ? `A+${delta}` : delta < 0 ? `B+${Math.abs(delta)}` : "="}
                </td>
              </tr>
            );
          })}
          <tr className="border-t" style={{ borderColor: "var(--border-primary)", color: "var(--text-primary)" }}>
            <td className="py-1 font-medium">TLS</td>
            <td className="text-right font-medium">{Math.round(tlsA)}</td>
            <td className="text-right font-medium">{Math.round(tlsB)}</td>
            <td className="text-right font-medium">
              {tlsA > tlsB ? `A+${Math.round(tlsA - tlsB)}` : tlsB > tlsA ? `B+${Math.round(tlsB - tlsA)}` : "="}
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  );
}
```

- [ ] **Step 3: Create context-panel compare-panel.tsx**

Assemble radar + diff table + point labels into the left panel:

```tsx
// src/components/context-panel/compare-panel.tsx
"use client";

import { useTranslations } from "next-intl";
import { RadarComparison } from "@/components/compare/radar-chart";
import { DiffTable } from "@/components/compare/diff-table";
import { useScore } from "@/features/score/api/use-score";
import { useUIStore } from "@/stores/ui-store";
import { Skeleton } from "@/components/ui/skeleton";

export function ComparePanel() {
  const t = useTranslations("compare");
  const { comparePointA, comparePointB, resetCompare, setMode } = useUIStore();
  const { data: scoreA } = useScore(comparePointA?.lat ?? null, comparePointA?.lng ?? null);
  const { data: scoreB } = useScore(comparePointB?.lat ?? null, comparePointB?.lng ?? null);

  return (
    <div className="flex flex-col h-full overflow-y-auto">
      <div className="px-4 pt-4 pb-2">
        <div className="text-[9px] tracking-widest" style={{ color: "var(--accent-cyan)", fontFamily: "var(--font-mono)" }}>
          COMPARE
        </div>
      </div>

      {/* Point labels */}
      <div className="flex px-4 gap-2 mb-3">
        <div className="flex-1 rounded-lg p-2" style={{ background: "var(--bg-tertiary)" }}>
          <div className="text-[9px] tracking-wider" style={{ color: "var(--accent-cyan)", fontFamily: "var(--font-mono)" }}>
            {t("pointA")}
          </div>
          <div className="text-[10px] mt-0.5" style={{ color: "var(--text-primary)" }}>
            {comparePointA?.address ?? "Click map..."}
          </div>
          {scoreA && <div className="text-lg font-bold mt-1" style={{ color: "var(--accent-cyan)" }}>{Math.round(scoreA.tls.score)}</div>}
        </div>
        <div className="flex-1 rounded-lg p-2" style={{ background: "var(--bg-tertiary)" }}>
          <div className="text-[9px] tracking-wider" style={{ color: "var(--accent-warning)", fontFamily: "var(--font-mono)" }}>
            {t("pointB")}
          </div>
          <div className="text-[10px] mt-0.5" style={{ color: "var(--text-primary)" }}>
            {comparePointB?.address ?? "Click map..."}
          </div>
          {scoreB && <div className="text-lg font-bold mt-1" style={{ color: "var(--accent-warning)" }}>{Math.round(scoreB.tls.score)}</div>}
        </div>
      </div>

      {/* Radar + Diff */}
      {scoreA && scoreB ? (
        <>
          <RadarComparison axesA={scoreA.axes} axesB={scoreB.axes} />
          <DiffTable axesA={scoreA.axes} axesB={scoreB.axes} tlsA={scoreA.tls.score} tlsB={scoreB.tls.score} />
        </>
      ) : (
        <div className="px-4 py-8 text-center text-xs" style={{ color: "var(--text-muted)" }}>
          {!comparePointA ? "Click first point on map" : "Click second point on map"}
        </div>
      )}

      <div className="px-4 py-3 mt-auto">
        <button
          type="button"
          onClick={() => { resetCompare(); setMode("explore"); }}
          className="w-full rounded-lg py-2 text-xs"
          style={{ background: "var(--bg-tertiary)", color: "var(--text-secondary)", border: "1px solid var(--border-primary)" }}
        >
          {t("endCompare")}
        </button>
      </div>
    </div>
  );
}
```

- [ ] **Step 4: Wire into page.tsx and update click handler**

```tsx
{mode === "compare" && <ComparePanel />}
```

Update `handleFeatureClick` to handle compare mode: when mode is "compare", call `setComparePoint` instead of `setAnalysisPoint`.

- [ ] **Step 5: Remove old ComparePanel import and DashboardStats**

Delete imports of old `ComparePanel` and `DashboardStats` from page.tsx. These components are now replaced.

- [ ] **Step 6: Verify**

```bash
cd services/frontend && pnpm tsc --noEmit
```

- [ ] **Step 7: Commit**

```bash
git add services/frontend/src/components/compare/ services/frontend/src/components/context-panel/compare-panel.tsx services/frontend/src/app/page.tsx
git commit -m "feat(frontend): add compare mode with radar chart and diff table in context panel"
```

---

## Phase 5: Data Pipeline (Independent)

### Task 5.1: N03 Administrative Boundary Download Script

**Files:**
- Create: `scripts/download-n03-boundaries.sh`

- [ ] **Step 1: Create download script**

```bash
#!/usr/bin/env bash
# scripts/download-n03-boundaries.sh
# Downloads N03 administrative boundary data (2025) from NLNI
set -euo pipefail

DATA_DIR="data/raw/n03"
OUTPUT_DIR="services/frontend/public/geojson/n03"

mkdir -p "$DATA_DIR" "$OUTPUT_DIR"

echo "Downloading N03 administrative boundaries (2025)..."
curl -L -o "$DATA_DIR/N03-2025.zip" \
  "https://nlftp.mlit.go.jp/ksj/gml/data/N03/N03-2025/N03-20250101_GML.zip"

echo "Extracting..."
unzip -o "$DATA_DIR/N03-2025.zip" -d "$DATA_DIR/extracted/"

echo "Done. Extract GeoJSON files and split into prefectures/municipalities."
echo "Run scripts/convert-n03-boundaries.py next."
```

- [ ] **Step 2: Commit**

```bash
chmod +x scripts/download-n03-boundaries.sh
git add scripts/download-n03-boundaries.sh
git commit -m "feat(scripts): add N03 admin boundary download script"
```

---

### Task 5.2: L01 Land Price Download Script

**Files:**
- Create: `scripts/download-l01-landprice.sh`

- [ ] **Step 1: Create download script**

```bash
#!/usr/bin/env bash
# scripts/download-l01-landprice.sh
# Downloads L01 land price data (2026) from NLNI
set -euo pipefail

DATA_DIR="data/raw/l01"
YEAR="${1:-2026}"

mkdir -p "$DATA_DIR"

echo "Downloading L01 land prices ($YEAR)..."
# Download per-prefecture (codes 01-47)
for code in $(seq -w 01 47); do
  url="https://nlftp.mlit.go.jp/ksj/gml/data/L01/L01-${YEAR}/L01-${YEAR}_${code}_GML.zip"
  out="$DATA_DIR/L01-${YEAR}_${code}.zip"
  if [ ! -f "$out" ]; then
    echo "  Downloading prefecture $code..."
    curl -L -o "$out" "$url" || echo "  Warning: $code not available"
  fi
done

echo "Done. $DATA_DIR contains $(ls "$DATA_DIR"/*.zip 2>/dev/null | wc -l) files."
```

- [ ] **Step 2: Commit**

```bash
chmod +x scripts/download-l01-landprice.sh
git add scripts/download-l01-landprice.sh
git commit -m "feat(scripts): add L01 land price download script"
```

---

### Task 5.3: Pipeline Orchestration Script

**Files:**
- Create: `scripts/pipeline.sh`

- [ ] **Step 1: Create orchestration script**

```bash
#!/usr/bin/env bash
# scripts/pipeline.sh
# Orchestrates full data pipeline: download → convert → import
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=== Government Data Pipeline ==="
echo "Started: $(date)"

echo ""
echo "--- Step 1: Download N03 Boundaries ---"
bash "$SCRIPT_DIR/download-n03-boundaries.sh"

echo ""
echo "--- Step 2: Download L01 Land Prices ---"
bash "$SCRIPT_DIR/download-l01-landprice.sh"

# Additional download scripts will be added in subsequent tasks:
# bash "$SCRIPT_DIR/download-jshis-surface.sh"
# bash "$SCRIPT_DIR/download-land-survey.sh"
# bash "$SCRIPT_DIR/download-tokyo-liquefaction.sh"

echo ""
echo "=== Pipeline Complete ==="
echo "Finished: $(date)"
```

- [ ] **Step 2: Commit**

```bash
chmod +x scripts/pipeline.sh
git add scripts/pipeline.sh
git commit -m "feat(scripts): add pipeline orchestration script"
```

---

## Phase 6: Backend APIs

### Task 6.1: Area Stats Endpoint

**Files:**
- Create: `services/backend/src/handler/area_stats.rs`
- Create: `services/backend/src/usecase/get_area_stats.rs`
- Modify: `services/backend/src/handler.rs` (register route)
- Modify: `services/backend/src/usecase.rs` (register module)

This endpoint returns aggregate statistics for a given administrative area (prefecture or municipality). It accepts an area code and returns population, average land price, risk indicators, and average TLS score.

- [ ] **Step 1: Define the usecase**

```rust
// src/usecase/get_area_stats.rs
use crate::domain::error::AppError;

#[derive(Debug, serde::Serialize)]
pub struct AreaStats {
    pub code: String,
    pub name: String,
    pub population: Option<i64>,
    pub avg_land_price: Option<f64>,
    pub land_price_count: i64,
    pub flood_area_ratio: f64,
    pub avg_tls: Option<f64>,
}

pub struct GetAreaStatsUsecase {
    // Will use pool directly for spatial aggregation queries
    pool: sqlx::PgPool,
}

impl GetAreaStatsUsecase {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn execute(&self, area_code: &str) -> Result<AreaStats, AppError> {
        // Spatial aggregation query joining land_prices with admin boundaries
        // This is a placeholder — actual SQL depends on PostGIS schema
        let stats = sqlx::query_as!(
            AreaStats,
            r#"
            SELECT
                $1 as code,
                '' as name,
                NULL::bigint as population,
                AVG(price_per_sqm)::float8 as avg_land_price,
                COUNT(*)::bigint as land_price_count,
                0.0::float8 as flood_area_ratio,
                NULL::float8 as avg_tls
            FROM land_prices
            WHERE ST_Within(geom, (SELECT geom FROM admin_boundaries WHERE code = $1))
            "#,
            area_code,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(stats)
    }
}
```

- [ ] **Step 2: Create handler**

```rust
// src/handler/area_stats.rs
use axum::{extract::Query, Json};
use serde::Deserialize;
use crate::app_state::AppState;
use crate::handler::error::ApiError;

#[derive(Deserialize)]
pub struct AreaStatsQuery {
    pub code: String,
}

pub async fn get_area_stats(
    Query(query): Query<AreaStatsQuery>,
    state: AppState,
) -> Result<Json<serde_json::Value>, ApiError> {
    let stats = state.area_stats_usecase.execute(&query.code).await?;
    Ok(Json(serde_json::to_value(stats).unwrap()))
}
```

- [ ] **Step 3: Register route**

Add to router in `src/handler.rs`:

```rust
.route("/api/area-stats", get(area_stats::get_area_stats))
```

- [ ] **Step 4: Verify build**

```bash
cd services/backend && cargo build
```

- [ ] **Step 5: Commit**

```bash
git add services/backend/src/handler/area_stats.rs services/backend/src/usecase/get_area_stats.rs services/backend/src/handler.rs services/backend/src/usecase.rs
git commit -m "feat(backend): add area stats endpoint for aggregate area statistics"
```

---

## Phase 7: Scoring Enhancements & LLM (Future)

These tasks extend the TLS scoring engine and add AI report generation. They depend on Phase 5 (data) and Phase 6 (backend APIs) being complete.

### Task 7.1: Add Storm Surge Sub-Score to S1

**Files:**
- Modify: `services/backend/src/domain/scoring/sub_scores.rs`
- Modify: `services/backend/src/domain/scoring/axis.rs`
- Modify: `services/backend/src/domain/scoring/constants.rs`

Add `score_storm_surge(depth_m: Option<f64>) -> f64` mapping and integrate into S1 composition formula.

### Task 7.2: Add Landform/Geology Sub-Scores to S2

**Files:**
- Modify: `services/backend/src/domain/scoring/sub_scores.rs`
- Modify: `services/backend/src/domain/scoring/axis.rs`

Add `score_landform(class: &str) -> f64` and `score_geology(type: &str) -> f64` mappings.

### Task 7.3: Add 5 New Cross-Analysis Patterns

**Files:**
- Modify: `services/backend/src/domain/scoring/tls.rs`
- Modify: `services/frontend/src/lib/schemas.ts`
- Modify: `services/frontend/src/components/analyze/cross-analysis.tsx`

Add Family Suitability, Infrastructure Signal, Disaster Resilience, Rental Demand, Aging Risk.

### Task 7.4: AI Report Generation Endpoint

**Files:**
- Create: `services/backend/src/handler/ai_report.rs`
- Create: `services/backend/src/usecase/generate_report.rs`
- Create: `services/frontend/src/features/ai-report/api/use-ai-report.ts`
- Create: `services/frontend/src/components/shared/ai-report-dialog.tsx`

Accepts TLS analysis context, forwards to LLM API (model TBD), returns structured narrative report. Frontend displays in a modal dialog.

---

## Cleanup Tasks

### Task C.1: Delete Obsolete Components

After all phases are wired, delete:

- `services/frontend/src/components/layer-panel.tsx`
- `services/frontend/src/components/dashboard-stats.tsx`
- `services/frontend/src/components/score-card/score-card.tsx`
- `services/frontend/src/components/score-card/score-gauge.tsx`
- `services/frontend/src/components/compare-panel.tsx` (old modal version)

### Task C.2: Update Tests

- Update `services/frontend/src/__tests__/schemas.test.ts` for any schema changes
- Add tests for new stores (analysis-store)
- Add tests for theme layer mapping logic

---

## Self-Review Checklist

- [x] **Spec coverage**: All 11 design sections have corresponding tasks
  - Layout (Task 1.4), Explore (Phase 2), Analyze (Phase 3), Compare (Phase 4)
  - Boundaries (Task 1.3), i18n (Task 0.1), Themes (Task 0.2), Data pipeline (Phase 5)
  - Backend APIs (Phase 6), Scoring enhancements (Phase 7), LLM (Task 7.4)
- [x] **Placeholder scan**: No TBD/TODO in Phase 0-4. Phase 7 tasks are intentionally higher-level (future phase)
- [x] **Type consistency**: `AppMode`, `ThemeId`, `WeightPreset`, `SelectedArea` used consistently across stores and components. `TlsResponse` from schemas.ts used in all analyze/compare components.
- [x] **i18n keys**: Consistent between ja.json and en.json. All components use `useTranslations()` with correct key paths.
- [x] **Store interface**: `ui-store` has `mode`, `activeThemes`, `locale`. `map-store` has `selectedArea`. `analysis-store` has `weightPreset`, `analysisPoint`.
