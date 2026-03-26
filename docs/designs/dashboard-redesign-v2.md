# Investment Analysis Dashboard — Redesign Specification v2.0

> Date: 2026-03-26
> Status: Design Complete, Ready for Implementation
> Previous: analysis-algorithm-design-integrated.html (TLS scoring engine)

---

## Table of Contents

1. [Vision & Goals](#1-vision--goals)
2. [Screen Layout & Mode Structure](#2-screen-layout--mode-structure)
3. [Explore Mode](#3-explore-mode)
4. [Analyze Mode](#4-analyze-mode)
5. [Compare Mode](#5-compare-mode)
6. [Administrative Boundaries & Area Filter](#6-administrative-boundaries--area-filter)
7. [i18n, Search, Settings, Data Pipeline](#7-i18n-search-settings-data-pipeline)
8. [Component Architecture & File Structure](#8-component-architecture--file-structure)
9. [Cross-Analysis Model Enhancements](#9-cross-analysis-model-enhancements)
10. [Decision Log](#10-decision-log)
11. [Risks & Mitigations](#11-risks--mitigations)

---

## 1. Vision & Goals

### What
Public data integration location analysis platform — aggregates 60+ datasets from 7 government portals to quantitatively evaluate location quality across 5 axes, with multilingual support.

### Why
- Government publishes vast primary data (disaster, terrain, infrastructure, demographics, land prices) but no tool exists for individuals to use it holistically
- Enable location judgments based on primary data, not financial market trends
- Establishing effective analysis/visualization patterns for public data is a goal in itself

### Who
- Individual investors — multi-axis property evaluation & comparison
- Relocation seekers — map-first exploration to discover good neighborhoods
- Data exploration — the platform itself as a public data utilization experiment

### Key Design Problems Solved
- **Information overload** → Theme presets auto-select relevant layers from 23+
- **No user journey** → 3 modes (Explore/Analyze/Compare) provide clear workflows
- **TLS engine buried** → Analysis mode puts 5-axis scoring front and center
- **No synthesis** → Cross-analysis patterns answer "so what?" questions
- **Dead dashboard** → Area cards with live data replace broken bottom stats

---

## 2. Screen Layout & Mode Structure

### Desktop Layout (1440px+)

```
+----------------------------------------------------------+
| [Mode: Explore|Analyze|Compare] [Search] [Settings] [EN/JA] |  Top Bar (48px)
+-------------+--------------------------------------------+
|             |                                            |
|  Context    |                MAP                         |
|  Panel      |          (remaining width)                 |
|  (320px)    |                                            |
|             |    Administrative boundaries always shown   |
|  Content    |    N03 prefectures + municipalities         |
|  depends    |                                            |
|  on mode    |                                            |
|             |                                            |
+-------------+--------------------------------------------+
| Status Bar (28px)                                        |
+----------------------------------------------------------+
```

### Context Panel Content by Mode

| Mode | Content |
|------|---------|
| Explore | Area card (selected prefecture/municipality stats) + Theme presets (Safety/Livability/Price/Future) + Layer settings (collapsed) |
| Analyze | TLS score header + 5-axis bars + Sub-score details (expandable) + Cross-analysis + Weight preset selector + AI report button |
| Compare | 2-point or 2-area parallel cards + Radar chart + Axis diff table + Summary |

### Components Removed

| Current Component | Replacement |
|---|---|
| `DashboardStats` (bottom panel, showing zeros) | `AreaCard` in explore mode |
| `LayerPanel` (23 toggles sidebar) | `ThemePresets` + `LayerSettings` (collapsed) |
| `ScoreCard` (right panel on click) | `AnalyzePanel` in left context panel |
| `ComparePanel` (fullscreen modal) | `ComparePanel` in left context panel |

---

## 3. Explore Mode

### Initial State (App Launch)

- Map shows Japan with all 47 prefecture + municipality boundaries (line-only, no fill)
- Context panel shows 4 theme preset buttons + collapsed layer settings
- Prompt: "Click a prefecture or municipality on the map"

### Theme Presets

| Theme | Auto-ON Layers | Map Visualization |
|-------|---------------|-------------------|
| Safety | Flood, tsunami, landslide, liquefaction, steep slope, seismic | Boundaries colored by S1 disaster risk (red→green) |
| Livability | Stations, schools, medical, parks, bus routes | Facility dot map + boundaries by access index |
| Price | Land price, land price time-series, zoning | Land price heatmap or 3D extrusion |
| Future | Population mesh, DID, location optimization | Boundaries by population change rate (blue→red) |

Themes are **multi-selectable** but default to single for first-time users.

### Area Click Behavior

1. Click prefecture → Highlight + zoom → Area card shows prefecture stats + municipality ranking
2. Click municipality within → Drill down → Area card updates to municipality stats
3. Click a data point (land price etc.) → Auto-transition to Analyze mode

### Area Card Contents

```
Area Card: Bunkyo-ku, Tokyo
  Population:    240,000
  Avg Land Price: ¥680,000/sqm
  Disaster Risk:  Medium
  Avg TLS:        68 (B)

  [Analyze this area →]
```

### Municipality Ranking

Displayed when a prefecture is selected. Sorted by selected theme's axis score.

```
Municipality Ranking (by TLS):
1. Chiyoda-ku  TLS 82 (A)
2. Minato-ku   TLS 79 (A)
3. Bunkyo-ku   TLS 76 (A)
...
```

---

## 4. Analyze Mode

### Entry Patterns

1. Click data point in Explore mode → Auto-transition with that point
2. Search bar address/station input → Direct launch
3. Manual mode tab switch → Last selected point or map center

### Layout

- **TLS Score Header**: Large score (72/100) + Grade (A) + Label ("Very Good")
- **Weight Preset Selector**: Dropdown — Balance / Investment / Residential / Disaster Prevention
- **5-Axis Bar Chart**: Color-coded bars with scores and confidence indicators
- **Axis Details (expandable)**: Each axis expands to show sub-scores with raw data values
- **Cross-Analysis**: 3-8 pattern results (Value Discovery, Demand Signal, Ground Safety, etc.)
- **AI Report Button**: Sends all analysis context to LLM for narrative report generation

### Axis Detail Expansion Example (S1 Disaster)

```
S1 Disaster Risk: 65 / 100
  Flood         ██████░░░░  50   Depth 0.5-3m (Sumida River)
  Liquefaction  ████████░░  80   PL class: Low
  Seismic 30yr  ███████░░░  55   Probability: 12%
  Tsunami       █████████░  90   Outside zone
  Landslide     ██████████ 100   Not applicable
  Storm surge   ███████░░░  70   Depth <0.3m

  ⚠ min() penalty: Flood (50) constrains overall score
  Formula: min(50,...) × weighted_avg / 100 = 65
```

### Weight Preset Real-time Recalculation

Switching preset instantly recalculates TLS, grade, and cross-analysis:

```
Balance → Disaster Prevention
  Disaster 25% → 40%
  Terrain  15% → 25%
  Livability 25% → 20%
  Future   15% → 5%
  Price    20% → 10%

  TLS: 72(A) → 63(B)
```

### AI Report Generation

Button triggers LLM with context:
- TLS scores (all axes + sub-scores + raw values)
- Area statistics (comparison vs municipal average)
- Land price trend (time-series data)
- Cross-analysis results

LLM returns:
- Summary evaluation (2-3 sentences)
- Highlights / risk points per axis
- Positioning vs surrounding areas
- Data limitation disclaimers

---

## 5. Compare Mode

### Two Comparison Types

| Type | Target | Trigger |
|------|--------|---------|
| Point comparison | 2 points' TLS full-axis | Click 2 points on map |
| Area comparison | 2 municipalities' aggregate stats | Select 2 from area ranking |

### Point Comparison Layout

- **Header**: Point A name + TLS vs Point B name + TLS
- **Radar Chart**: 5-axis overlay (A=cyan, B=amber)
- **Axis Diff Table**: Per-axis scores with delta and winner indicator
- **Cross-Analysis Comparison**: Side-by-side pattern values
- **Summary**: "A excels in livability/safety; B excels in price/terrain"

### Area Comparison

Same structure but with aggregate values:

```
Area Comparison:
              Bunkyo-ku   Suginami-ku   Delta
Population    240k        591k          Suginami 2.5x
Avg Price     ¥680k       ¥420k         Bunkyo +62%
Avg TLS       68 (B)      62 (B)        Bunkyo +6
Flood area    12%         5%            Suginami safer
Station density 3.2/km²  2.1/km²       Bunkyo better
Vacancy rate  11%         13%           Bunkyo lower
```

### Map Visualization

- Single map (no split view) with both points/areas visible
- 2 pins with 500m radius circles for point comparison
- 2 highlighted municipality polygons for area comparison

---

## 6. Administrative Boundaries & Area Filter

### Display Rules (Always Visible)

| Zoom | Display | Style |
|------|---------|-------|
| z4-z7 | Prefectures only | White 1px, opacity 0.6 |
| z7-z10 | Prefectures + municipalities | Prefecture: white 1.5px / Municipality: white 0.5px, opacity 0.4 |
| z10+ | Municipalities (prefectures thicker) | Municipality: white 1px / Prefecture: white 2px |

### Labels

| Zoom | Labels |
|------|--------|
| z4-z7 | Prefecture names |
| z8-z10 | Prefecture + major municipality names |
| z11+ | Municipality names only |

### Area Selection Behavior

```
Click boundary polygon
  → Identify prefecture or municipality
  → Highlight (cyan stroke 2px + fill opacity 0.1)
  → Context panel shows area card
  → Theme layers filter to that area's data

Within selected prefecture, click municipality → Drill down
Within selected municipality, click another → Switch selection
Within selected municipality, click data point → Transition to Analyze mode
ESC or click outside or breadcrumb up → Deselect
```

### Breadcrumb Navigation

```
Japan > Tokyo > Bunkyo-ku
        ↑ Click to return to Tokyo level
```

### Data Filtering

```
No area selected:    bbox = current viewport
Prefecture selected: bbox = prefecture bounding box + prefCode filter
Municipality selected: bbox = municipality bounding box + cityCode filter
```

Applied to both REINFOLIB API calls and backend API queries.

---

## 7. i18n, Search, Settings, Data Pipeline

### i18n

- Library: next-intl (App Router native)
- Languages: Japanese (default), English
- URL state: `?lang=en` via nuqs
- Translation scope: UI labels, layer names, TLS grades, analysis templates, admin area names (mapping table for EN names)
- AI reports: Language specified in LLM prompt

### Search Bar

Top bar center. Address / station / municipality search.

```
Input: "Bunkyo Hongo"
  Candidates (dropdown):
    📍 3-chome Hongo, Bunkyo-ku, Tokyo — Address
    🏛 Bunkyo-ku — Area
    🚉 Hongo-sanchome Station — Station

Selection:
  Address → Analyze mode at that point
  Area → Explore mode with that area selected
  Station → Analyze mode at station coordinates
```

Implementation: REINFOLIB XIT002 (municipality list) + static station data + geocoding API (GSI or Nominatim)

### Settings

```
Settings drawer:
  ─── Display ───
  Language: [Japanese ▼]
  Map style: [Dark ▼] (Dark / Light / Satellite)

  ─── Analysis ───
  Default radius: [500m ▼] (300m / 500m / 1km)
  Weight preset: [Balance ▼]

  ─── Data ───
  Cache updated: 2026-03-25
  [Manual refresh] button
```

### Data Pipeline

```
Weekly cron batch:
  scripts/download-*.sh     → data/raw/{source}/
  scripts/convert-*.py      → data/processed/geojson/
  scripts/import-to-postgis.py → PostgreSQL + PostGIS

REINFOLIB API cache:
  Rust backend → SQLite (24h TTL, existing mechanism)
  Area stats → Computed results cached in PostgreSQL

Frontend static data:
  N03 boundaries → public/geojson/n03/ (split by prefecture)
  Pre-computed theme scores → public/geojson/scores/
```

### Scripts (scripts/ directory)

```
scripts/
  download-n03-boundaries.sh      # N03 admin boundaries (nationwide)
  download-l01-landprice.sh       # L01 land prices (44 years)
  download-jshis-surface.sh       # J-SHIS surface geology
  download-land-survey.sh         # Landform/geology/soil classification
  download-tokyo-liquefaction.sh  # Tokyo Metropolitan liquefaction
  download-disaster-history.sh    # Disaster history records
  download-plateau-tokyo.sh       # PLATEAU Tokyo 23 wards
  convert-gml-to-geojson.py       # GML → GeoJSON
  convert-shp-to-geojson.py       # SHP → GeoJSON
  import-to-postgis.py            # PostGIS import
  fetch-reinfolib-cache.py        # REINFOLIB API batch fetch
  fetch-estat-census.py           # e-Stat census data
  pipeline.sh                     # Orchestration (runs all above)
```

---

## 8. Component Architecture & File Structure

### New File Structure

```
src/
  app/
    page.tsx                      # Mode router (thin orchestrator)
    layout.tsx                    # Existing + i18n Provider

  components/
    top-bar/
      top-bar.tsx                 # Mode tabs + search + settings + locale
      mode-tabs.tsx               # Explore | Analyze | Compare
      search-bar.tsx              # Address/station/area search
      locale-toggle.tsx           # JA/EN toggle

    context-panel/
      context-panel.tsx           # Mode-dependent panel container
      explore-panel.tsx           # Explore: themes + area card
      analyze-panel.tsx           # Analyze: TLS full axis + cross
      compare-panel.tsx           # Compare: 2-point/area comparison

    explore/
      theme-presets.tsx           # Safety/Livability/Price/Future buttons
      area-card.tsx               # Area statistics card
      area-ranking.tsx            # Municipality TLS ranking
      breadcrumb-nav.tsx          # Japan > Tokyo > Bunkyo-ku

    analyze/
      tls-score-header.tsx        # Large score + grade display
      axis-bar-list.tsx           # 5-axis bar chart list
      axis-detail.tsx             # Expanded sub-score detail
      weight-preset-selector.tsx  # Weight preset switcher
      cross-analysis.tsx          # Cross-analysis results (3-8 patterns)
      ai-report-button.tsx        # LLM report trigger

    compare/
      point-comparison.tsx        # 2-point parallel cards
      area-comparison.tsx         # 2-area aggregate comparison
      radar-chart.tsx             # Radar chart (existing, modified)
      diff-table.tsx              # Axis diff table

    map/
      map-view.tsx                # Existing, maintained
      layers/                     # Existing 23 layers, maintained
        boundary-layer.tsx        # [NEW] N03 admin boundary layer
        theme-choropleth.tsx      # [NEW] Theme color-fill layer
      area-highlight.tsx          # [NEW] Selected area highlight

    shared/
      layer-settings.tsx          # Traditional 23-layer toggles (collapsed)
      settings-drawer.tsx         # Settings panel
      ai-report-dialog.tsx        # LLM report display modal

    ui/                           # shadcn/ui, existing
    status-bar.tsx                # Existing, maintained

  stores/
    map-store.ts                  # Existing + selectedArea
    ui-store.ts                   # Existing → mode: 'explore'|'analyze'|'compare'
    analysis-store.ts             # [NEW] Weight preset, analysis point, results

  features/
    area-stats/                   # [NEW] Area aggregate statistics
      api/use-area-stats.ts
    tls-score/                    # Existing score, renamed
      api/use-tls-score.ts
    search/                       # [NEW] Address/station search
      api/use-search.ts
    ai-report/                    # [NEW] LLM report
      api/use-ai-report.ts
    boundary/                     # [NEW] Admin boundaries
      api/use-boundaries.ts

  i18n/
    locales/ja.json
    locales/en.json
    config.ts

  hooks/
    use-active-mode.ts            # [NEW] Current mode + transition logic
    use-area-selection.ts         # [NEW] Area selection state
```

### State Management

| Store | Responsibility | Key State |
|-------|---------------|-----------|
| `map-store` | Map view + layer visibility | viewState, visibleLayers, selectedFeature, selectedArea |
| `ui-store` | UI mode + panel state | mode, layerSettingsOpen, locale |
| `analysis-store` | Analysis parameters | weightPreset, analysisPoint, comparePoints, analysisRadius |

### TanStack Query Key Design

```ts
const keys = {
  tlsScore: (lat, lng, preset) => ['tls', lat, lng, preset],
  areaStats: (code, theme) => ['area-stats', code, theme],
  ranking: (parentCode, sortAxis) => ['ranking', parentCode, sortAxis],
  boundaries: (level, parentCode) => ['boundaries', level, parentCode],
  search: (query) => ['search', query],
}
```

---

## 9. Cross-Analysis Model Enhancements

### Existing Patterns (3)

```
Value Discovery  = S1 × (100 - V_rel) / 100
Demand Signal    = S3 × S4 / 100
Ground Safety    = S1 × S2 / 100
```

### New Patterns (5)

```
Family Suitability = school_district_quality×0.3 + nursery_access×0.2
                     + park_area×0.2 + S1×0.3

Infrastructure Signal = urban_plan_road×0.3 + location_optimization×0.3
                        + ridership_trend×0.2 + pop_trend×0.2

Disaster Resilience = S1 × S2 × (1 - disaster_history_freq×0.1)
                      × shelter_proximity

Rental Demand = station_proximity×0.3 + pop_density×0.2
                + commercial_density×0.2 + vacancy_inverse×0.3

Aging Risk = elderly_ratio × (1 - medical_access) × (1 - welfare_access)
```

### Axis Enhancements

**S1 (Disaster)**: Add storm surge (XKT027), large fill risk (XKT020)
**S2 (Terrain)**: Add landform classification, geology, elevation mesh
**S3 (Livability)**: Add nursery/kindergarten, bus routes, welfare, libraries
**S4 (Future)**: Add vacancy rate, compact city zones, planned roads
**S5 (Price)**: Add REINS contract prices, PLATEAU building attributes

See `docs/research/2026-03-26-government-data-sources-comprehensive.md` for full data source details.

---

## 10. Decision Log

| # | Decision | Alternatives | Rationale |
|---|----------|-------------|-----------|
| D1 | 3-mode structure (Explore/Analyze/Compare) | A: Area drilldown only, C: Dashboard grid | Covers all use cases. A's strength absorbed into Explore mode. C sacrifices map experience |
| D2 | Single left context panel for all modes | Left + right 2-panel | Mode context changes make 1 panel sufficient. Right panel takes map space |
| D3 | 4 theme presets auto-control 23 layers | Individual toggles only (current) | New users don't struggle with "which to turn on." Power users get collapsed toggles |
| D4 | DashboardStats removed → AreaCard | Keep bottom panel | Current shows all zeros. Area-linked stats are meaningful |
| D5 | N03 boundaries always visible (pref + muni) | Zoom-dependent single level | User requirement. Adjust line weight/opacity/labels by zoom |
| D6 | Area click → data filtering | Explicit filter UI | Intuitive spatial UX. Breadcrumb shows current scope |
| D7 | REINFOLIB API as primary data source | NLNI static only | 31 APIs cover nearly all TLS sub-scores. API key already configured |
| D8 | Weekly batch + SQLite cache | Real-time API calls | User specification. Reuse existing 24h TTL SQLite mechanism |
| D9 | next-intl for i18n | next-i18next, manual | App Router native. Server Component compatible. Start with JA/EN |
| D10 | No split view in Compare mode | Left/right split map | Single map better for spatial relationship understanding |
| D11 | LLM model selection deferred | Decide now | User specification. Design UI/prompts first, model swappable later |
| D12 | Add 5 cross-analysis patterns (total 8) | Keep 3 | New data sources (nursery, bus, vacancy, PLATEAU) enable richer analysis |
| D13 | Enhance S1-S4 with new sub-scores | Keep current sub-scores | REINFOLIB APIs provide storm surge, fill risk, etc. Land survey provides landform/geology |

---

## 11. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| N03 boundary data 603MB initial load | Display latency | Split by prefecture + vectorize with tippecanoe |
| REINFOLIB API monthly request limit | Batch fetch throttled | Weekly batch + cache minimizes API calls |
| Area aggregation API needs new backend endpoint | Backend dev required | Existing PostGIS + spatial aggregation SQL |
| 23 layers → theme auto-control migration | Layer compatibility | Layer components maintained as-is. Only visibleLayers control changes |
| i18n for admin area names (no EN in N03) | Missing English names | Maintain separate JA→EN mapping table for prefectures + major municipalities |

---

## References

- TLS Scoring Engine: `docs/designs/analysis-algorithm-design-integrated.html`
- Government Data Research: `docs/research/2026-03-26-government-data-sources-comprehensive.md`
- REINFOLIB API Spec: `docs/research/2026-03-21-reinfolib-api-spec.md`
- Current Data Inventory: `docs/research/2026-03-23-data-inventory.md`
