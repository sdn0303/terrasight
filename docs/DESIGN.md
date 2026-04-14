# Terrasight Design System

## 1. Design Philosophy

Terrasight presents real estate investment data through a clean, information-first interface inspired by mapleads and rakumachi. The layout centers on a full-screen map with overlaid panels that slide in contextually -- no persistent chrome that competes with the data.

The visual language draws from mapleads: white panel surfaces, generous border-radius (12-16px), soft drop shadows, and an indigo/blue accent system. Panels appear only when the user requests them; the default state is map-dominant.

Information architecture follows rakumachi's thematic model: instead of toggling individual layers, users switch between exclusive themes (地価, ハザード, 取引事例, etc.). Each theme activates a curated set of layers and surfaces theme-linked detail panels.

**Multi-theme map support**: Light (default), Dark, and Satellite map styles are available via a Map Style Switcher.

Key design principles:
- Map-first: the map fills the viewport at all times; panels overlay without replacing it
- Theme-based exclusivity: activating one theme deactivates the previous one (rakumachi pattern)
- Progressive disclosure: detail panels appear only on user action (click, table row selection)
- Clean surfaces: white backgrounds, soft shadows `rgba(0,0,0,0.08)`, no hard borders on light mode

## 2. Color Tokens

All color tokens are defined as CSS custom properties in `:root` scope.

Source of truth: `services/frontend/src/app/globals.css`

Palette constants are sourced from `src/lib/palette.ts`. CSS variables in `globals.css` and Mapbox paint expressions both derive from this file.

### Base Palette (Light-first with Dark Mode Support)

| Token | Light Value | Dark Value | Usage |
|---|---|---|---|
| `--bg-primary` | `#FFFFFF` | `#0c0c14` | Panel backgrounds, page background |
| `--bg-secondary` | `#F9FAFB` | `#13131e` | Secondary surfaces, input backgrounds |
| `--bg-tertiary` | `#F3F4F6` | `#1a1a28` | Hover states, muted surfaces |
| `--text-primary` | `#111827` | `#e4e4e7` | Body text, data values |
| `--text-secondary` | `#6B7280` | `#a1a1aa` | Labels, descriptions |
| `--text-muted` | `#9CA3AF` | `#52525b` | Disabled text, category headers |
| `--text-heading` | `#030712` | `#f4f4f5` | Panel titles, headings |
| `--border-primary` | `rgba(0,0,0,0.08)` | `rgba(63,63,70,0.5)` | Panel borders, dividers |
| `--shadow-panel` | `rgba(0,0,0,0.08)` | `rgba(0,0,0,0.4)` | Panel drop shadows |

### Accent Colors

| Token | Value | Usage |
|---|---|---|
| `--accent-indigo` | `#6366F1` | Primary accent, active indicators, focus rings |
| `--accent-indigo-tint` | `rgba(99,102,241,0.12)` | Active item highlight (sidebar, table row) |
| `--hover-blue-tint` | `rgba(59,130,246,0.06)` | Hover state background |
| `--accent-danger` | `#e04030` | Destructive actions, high-risk indicators |
| `--accent-warning` | `#ffd000` | Warnings, mid-risk indicators |
| `--accent-success` | `#10b981` | Success states |

### shadcn/ui Theme Mapping

These tokens bridge the custom palette to shadcn/ui component internals (light mode defaults):

| Token | Value | Maps to |
|---|---|---|
| `--background` | `#FFFFFF` | Same as `--bg-primary` |
| `--foreground` | `#111827` | Same as `--text-primary` |
| `--card` / `--card-foreground` | `#FFFFFF` / `#111827` | Card surfaces |
| `--popover` / `--popover-foreground` | `#FFFFFF` / `#111827` | Tooltip/popover surfaces |
| `--primary` / `--primary-foreground` | `#6366F1` / `#FFFFFF` | Indigo-on-white for primary buttons |
| `--secondary` / `--secondary-foreground` | `#F3F4F6` / `#111827` | Secondary surface buttons |
| `--muted` / `--muted-foreground` | `#F9FAFB` / `#6B7280` | Muted backgrounds and text |
| `--accent` / `--accent-foreground` | `rgba(99,102,241,0.12)` / `#111827` | Accent surface |
| `--destructive` | `#e04030` | Destructive variant |
| `--border` | `rgba(0,0,0,0.08)` | All borders |
| `--input` | `rgba(0,0,0,0.08)` | Input borders |
| `--ring` | `#6366F1` | Focus ring |
| `--radius` | `0.75rem` | Base border-radius (12px) |

### Layer Color Tokens

Each data layer has a dedicated color token used for the layer indicator in the panel and the Mapbox paint expression. These values are intentional raw hex constants -- Mapbox paint expressions do not support CSS custom properties.

| Token | Hex | Layer |
|---|---|---|
| `--layer-landprice` | `#fbbf24` | Land Price (地価公示) |
| `--layer-flood-history` | `#60a5fa` | Flood History (浸水履歴) |
| `--layer-did` | `#a78bfa` | DID Area (人口集中地区) |
| `--layer-station` | `#f472b6` | Railway Stations (鉄道駅) |
| `--layer-flood` | `#0ea5e9` | Flood Risk (洪水浸水) |
| `--layer-steep-slope` | `#f97316` | Steep Slope (急傾斜地) |
| `--layer-liquefaction` | `#eab308` | Liquefaction Risk (液状化危険度) |
| `--layer-seismic` | `#ef4444` | Seismic Hazard (地震動・震源断層) |
| `--layer-fault` | `#ef4444` | Fault Lines (断層線) |
| `--layer-volcano` | `#f43f5e` | Volcanoes (火山) |
| `--layer-landslide` | `#fb923c` | Landslide Risk (土砂災害) |
| `--layer-tsunami` | `#38bdf8` | Tsunami Risk (津波浸水) |
| `--layer-landform` | `#d4a574` | Landform (地形分類) |
| `--layer-geology` | `#8b7355` | Geology (表層地質) |
| `--layer-soil` | `#a0845c` | Soil (土壌図) |
| `--layer-schools` | `#34d399` | Schools (学校) |
| `--layer-medical` | `#2dd4bf` | Medical (医療機関) |
| `--layer-school-dist` | `#4ade80` | School Districts (小学校区) |
| `--layer-park` | `#86efac` | Parks (都市公園) |
| `--layer-railway` | `#22d3ee` | Railway Lines (鉄道路線) |
| `--layer-boundary` | `#a1a1aa` | Admin Boundary (市町村境界) |
| `--layer-zoning` | `#818cf8` | Zoning (用途地域) |
| `--layer-population` | `#c084fc` | Population Mesh (将来人口メッシュ) |
| `--layer-urban-plan` | `#34d399` | Urban Planning Zones (立地適正化) |

Color heuristics by category:
- **Value layers**: warm yellows, blues, pinks
- **Risk layers**: reds, oranges, sky blues (danger spectrum)
- **Ground layers**: earth tones (tan, brown)
- **Infrastructure layers**: greens, teals, cyan
- **Orientation layers**: neutrals, purples, greens

## 3. Typography

Source of truth: `services/frontend/src/app/layout.tsx`, `globals.css`

### Font Stack

| Token | Value | Usage |
|---|---|---|
| `--font-sans` | `"Inter", system-ui, sans-serif` | Headings, panel labels, body text |
| `--font-mono` | `monospace, system-ui` | Data values, coordinates, numeric readouts |

Inter is the primary typeface for all UI chrome. Monospace is reserved for dense data contexts (coordinates, price values, status readouts) where alignment and fixed-width rendering aid scannability.

### Type Scale (in use)

| Context | Size | Weight | Tracking | Font |
|---|---|---|---|---|
| Sidebar nav item | `text-sm` (14px) | `font-medium` | Default | Sans |
| Panel section header | `text-xs` (12px) | `font-semibold` | `0.08em` | Sans |
| Panel body / data label | `text-sm` (14px) | Normal | Default | Sans |
| Data value (price, score) | `text-sm` (14px) | `font-medium` | Default | Mono |
| Table cell | `text-xs` (12px) | Normal | Default | Sans |
| Map legend label | `10px` | Normal | Default | Sans |
| Status bar | `10px` | Normal | Default | Mono |
| Coordinate readout | `11px` | Normal | Default | Mono |

## 4. Component Patterns

### shadcn/ui Components

The project uses shadcn/ui (style: `new-york`) with the following components:

- **Button** (`button.tsx`) -- standard variants
- **Card** (`card.tsx`) -- white background, 12-16px border-radius, `shadow-sm`
- **Collapsible** (`collapsible.tsx`) -- sidebar section expand/collapse
- **ScrollArea** (`scroll-area.tsx`) -- scrollable panels and drawers
- **Separator** (`separator.tsx`) -- dividers within panels
- **Sheet** (`sheet.tsx`) -- Right Drawer (slides from right)
- **Skeleton** (`skeleton.tsx`) -- loading placeholders in panels and table
- **Tabs** (`tabs.tsx`) -- tab navigation within Left Detail Panel and Right Drawer
- **Toggle** (`toggle.tsx`) -- map style switcher, layer toggles
- **Tooltip** (`tooltip.tsx`) -- wrapped at root via `TooltipProvider`

### AppShell

The root layout component. Renders the full-screen Mapbox map as the base layer with all panels positioned as fixed overlays via CSS `position: fixed`. Manages the four layout states (see Section 5) by coordinating Zustand store flags for `leftPanelOpen`, `tableOpen`, and `rightDrawerOpen`.

### Sidebar

mapleads-style collapsible navigation rail.

- **Expanded**: 200px wide, shows icon + label for each nav item
- **Collapsed**: 56px wide, shows icon only with tooltip
- Collapse toggle at the bottom of the rail
- Two sections: "探す" (search/browse -- themes) and "見る" (analysis -- Opportunities)
- Active item background: `rgba(99,102,241,0.12)` (indigo tint)
- Hover background: `rgba(59,130,246,0.06)` (blue tint)
- White panel surface, soft shadow `rgba(0,0,0,0.08)`, border-radius 12px on the right edge
- z-index: 40

**Nav items (探す):**
- 地価 (Land Price)
- ハザード (Hazard)
- 取引事例 (Transaction Cases)
- 乗降客数 (Station Ridership)
- スコア分析 (Score Analysis)

**Nav items (見る):**
- Opportunities (opens Opportunities Table)

### Left Detail Panel

Slides in from the left when a theme is active and the user clicks the map (State 1).

- Width: 360px
- Full height (minus status bar)
- White background, 0 12px 12px 0 border-radius, soft shadow
- Tab navigation matching the active theme (rakumachi style -- tabs are theme-linked)
- z-index: 60
- Animation: `transform: translateX(-100%)` → `translateX(0)` over 0.3s ease

### Opportunities Table

On-map overlay table. Opens when the user clicks the Opportunities nav item.

- Width: ~65% of viewport width, centered horizontally
- Positioned at the bottom of the viewport, slides up from below
- Virtualized rows (TanStack Virtual) for performance
- mapleads CRM-style: compact rows, sticky header, sortable columns
- Active row background: `rgba(99,102,241,0.12)`
- White surface, top border-radius 12px, shadow above
- z-index: 80
- Clicking a row opens the Right Drawer (State 3)

### Right Drawer

340px panel slides in from the right. Opens in two contexts:
1. Table row click -- shows Opportunity detail
2. Map point click while table is open (State 3) -- shows map point detail

- Width: 340px
- Full height (minus status bar)
- White background, 12px 0 0 12px border-radius, soft shadow
- Contains tabs: Detail / Compare (replaces old ComparePanel modal)
- z-index: 100
- Animation: `transform: translateX(100%)` → `translateX(0)` over 0.3s ease

### Map Style Switcher

Three-state toggle for base map style.

- Options: Light (streets-v12), Dark (dark-v11), Satellite (satellite-streets-v12)
- Positioned top-right of the map, above NavControl
- Compact toggle group (shadcn/ui `Toggle`)
- z-index: 20

### StatusBar

A fixed 28px bar at the bottom of the viewport. Monospaced, 10px. Displays coordinates, zoom level, and loading state. Simplified from previous version -- no DEMO badge.

- z-index: 20

### Map Paint Colors (Mapbox Exception)

Mapbox GL paint expressions do not support CSS custom properties -- they require raw hex values. The `--layer-*` hex tokens listed in Section 2 are the canonical source. If the brand palette changes, both `globals.css` and `src/lib/palette.ts` must be updated; Mapbox paint expressions in layer components will pick up values from `palette.ts` at build time via the shared constants.

**Land Price Color Ramp** (source: `land-price-layer.tsx`):

| Hex | Price Range (¥/m²) | Label |
|---|---|---|
| `#3b82f6` (blue) | ~100,000 | Low |
| `#22c55e` (green) | ~300,000 | Mid |
| `#eab308` (yellow) | ~500,000 | High |
| `#ef4444` (red) | ~1,000,000 | Very high |
| `#a855f7` (purple) | ~3,000,000+ | Premium |

## 5. Layout Structure

### Viewport States

The AppShell cycles through four discrete layout states:

```
State 0: Map only (initial)
+----+------------------------------------------+
| SB |           Mapbox Map (full)              |
|    |                                          |
| 56 |                                          |
| px |                       [Style Switcher]   |
|    |                       [NavControl]       |
+----+------------------------------------------+
| StatusBar (28px)                              |
+-----------------------------------------------+

State 1: Nav + Left Panel + Map
+----+--------+--------------------------------+
| SB | Left   |      Mapbox Map               |
|    | Panel  |                               |
|    | 360px  |                               |
|    |        |              [Style Switcher] |
|    |        |              [NavControl]     |
+----+--------+--------------------------------+
| StatusBar (28px)                             |
+----------------------------------------------+

State 2: Nav + Table + Map
+----+------------------------------------------+
| SB |           Mapbox Map (full)              |
|    |                                          |
|    |  +------------------------------------+  |
|    |  | Opportunities Table (~65% w)       |  |
|    |  | [virtualized rows]                 |  |
+----+--+------------------------------------+--+
| StatusBar (28px)                              |
+-----------------------------------------------+

State 3: Nav + Table + Right Drawer + Map
+----+-----------------------------+----------+
| SB |      Mapbox Map             |  Right   |
|    |                             |  Drawer  |
|    |  +--------------------+    |  340px   |
|    |  | Opportunities Table|    |          |
|    |  | (~65% w)           |    |          |
+----+--+--------------------+----+----------+
| StatusBar (28px)                            |
+---------------------------------------------+
```

SB = Sidebar (56px collapsed / 200px expanded)

### Key Dimensions

| Element | Size |
|---|---|
| Sidebar (collapsed) | 56px wide, full height minus status bar |
| Sidebar (expanded) | 200px wide, full height minus status bar |
| Left Detail Panel | 360px wide, full height minus status bar |
| Opportunities Table | ~65% viewport width, bottom-anchored |
| Right Drawer | 340px wide, full height minus status bar |
| Status bar | 28px tall, full width |
| Panel border-radius | 12-16px |
| Layer indicator dot | 8px (w-2 h-2) |

### z-index Stack

| z-index | Element |
|---|---|
| 100 | Right Drawer |
| 80 | Opportunities Table |
| 60 | Left Detail Panel |
| 40 | Sidebar |
| 20 | Map controls / Legend / Style Switcher / StatusBar |
| 1 | Mapbox Map |

## 6. Map Configuration

Source of truth: `services/frontend/src/features/map/components/MapView.tsx`

| Property | Value |
|---|---|
| Library | Mapbox GL JS (via react-map-gl) |
| Token env var | `NEXT_PUBLIC_MAPBOX_TOKEN` |
| Default style | `mapbox://styles/mapbox/streets-v12` (light) |
| Dark style | `mapbox://styles/mapbox/dark-v11` |
| Satellite style | `mapbox://styles/mapbox/satellite-streets-v12` |
| Default center | `[139.767, 35.681]` (Tokyo) |
| Default zoom | 12 |
| Default pitch | 0 degrees (flat -- optimized for data readability) |
| Default bearing | 0 |
| 3D buildings | Optional, disabled by default |
| Terrain | Optional, disabled by default |
| Move debounce | 300ms |

3D buildings and terrain are available as opt-in overlays via the Map Style Switcher but are not activated by default. Flat view (pitch: 0) is the default for investment data readability.

## 7. Theme System (replaces Layer Toggle System)

Themes replace the 24-layer toggle system. Activating a theme switches the active layer set exclusively -- the previous theme's layers are hidden before the new ones appear (0.3s fade transition).

Source of truth: `services/frontend/src/lib/themes.ts` and `docs/designs/map-visualization-spec.md`

### Available Themes

| Theme ID | Japanese | Primary Layers Activated |
|---|---|---|
| `landprice` | 地価 | Land Price, Admin Boundary, Zoning |
| `hazard` | ハザード | Flood, Liquefaction, Seismic, Landslide, Tsunami, Steep Slope |
| `transactions` | 取引事例 | Transaction Case points, Zoning |
| `ridership` | 乗降客数 | Station Ridership circles, Railway Lines |
| `score` | スコア分析 | Composite score mesh, Admin Boundary |

Each theme's exact layer set and paint expressions are specified in `docs/designs/map-visualization-spec.md`.

### Layer ID Conventions

- UI layer IDs: `underscore_case` (e.g., `land_price`, `flood_risk`)
- WASM/FlatGeobuf layer IDs: `hyphen-case` (e.g., `land-price`, `flood-risk`)
- Use `canonicalLayerId()` from `src/lib/layers.ts` when crossing the boundary

## 8. Interaction Specification

### Theme Switching

- User clicks a theme item in the Sidebar
- Active theme flag updates in Zustand store
- Previous theme layers fade out (opacity 0, 0.3s)
- New theme layers fade in (opacity 1, 0.3s)
- Left Detail Panel closes if open (reset to State 0 or State 2)

### Map Click in State 0 (map only)

- User clicks a map feature
- Left Detail Panel slides in (State 0 → State 1)
- Panel shows theme-linked detail tabs for the clicked location

### Map Click in State 2 (table open)

- User clicks a map feature while Opportunities Table is open
- Right Drawer slides in (State 2 → State 3)
- Drawer shows point detail for the clicked feature

### Table Row Click

- User clicks a row in the Opportunities Table
- Right Drawer slides in (State 2 → State 3)
- Drawer shows Opportunity detail with tabs: Detail / Compare

### Opportunities Nav Click

- User clicks "Opportunities" in the Sidebar
- If Left Detail Panel is open, it closes first
- Opportunities Table slides up from bottom (State 0/1 → State 2)

### Sidebar Collapse/Expand

- User clicks the collapse toggle
- Sidebar width animates between 200px and 56px (0.2s ease)
- Map canvas does not reflow -- sidebar overlays the map

### Map Style Switch

- User selects Light / Dark / Satellite from Map Style Switcher
- `map.setStyle()` is called; layer sources and paint expressions are re-applied on the `style.load` event

## 9. Accessibility

- All interactive overlay components are keyboard-navigable (Tab, Enter, Escape)
- Sidebar nav items carry `aria-label` with the theme name
- Left Detail Panel and Right Drawer are implemented as `role="complementary"` regions with `aria-label`
- Opportunities Table uses `role="grid"` with `aria-rowcount` for virtualized rows
- Focus is trapped within the Right Drawer when open (shadcn/ui Sheet behavior)
- Map canvas carries `aria-label="Interactive map"` and `role="application"`
- Color is never the sole indicator of meaning -- layer icons and text labels accompany all color-coded elements
- WCAG AA contrast is required for all text on panel backgrounds
- `prefers-reduced-motion`: panel slide animations are disabled; state transitions use opacity-only fade

## 10. Loading and Error States

### Loading

- Left Detail Panel: Skeleton rows while fetching theme-linked data
- Right Drawer: Skeleton layout matching Opportunity detail shape
- Opportunities Table: First 20 rows shown as Skeleton while initial fetch resolves
- Map layers: Mapbox source loading is handled natively; no additional spinner
- StatusBar displays a loading indicator (monospace dot animation) during active fetches

### Error States

- Panel fetch error: inline error message with a retry button; panel stays open
- Map source error: toast notification (shadcn/ui `Sonner`) at top-right; map remains interactive
- Opportunities Table fetch error: empty state with retry CTA
- All error boundaries implemented via React `error.tsx` files per the Next.js App Router convention

## 11. Domain Model (Frontend)

Source of truth: `services/frontend/src/features/*/types/`

### Opportunity

The core investable unit. An Opportunity aggregates a land parcel with zoning, hazard scores, and transaction history.

| Field | Type | Notes |
|---|---|---|
| `id` | `string` | UUID |
| `coordinates` | `[number, number]` | `[lng, lat]` (RFC 7946) |
| `address` | `string` | Human-readable address |
| `ward` | `string` | Tokyo 23 ward name |
| `landPrice` | `number \| null` | ¥/m², latest available year |
| `zoningCode` | `string` | Zoning category code |
| `floodRiskLevel` | `number \| null` | 0-4 ordinal scale |
| `compositeScore` | `number \| null` | 0-100 investment score |
| `transactionCount` | `number` | Number of recorded transactions |
| `createdAt` | `string` | ISO 8601 |

### ThemeState (Zustand)

| Field | Type | Notes |
|---|---|---|
| `activeTheme` | `ThemeId \| null` | Currently active theme |
| `mapStyle` | `'light' \| 'dark' \| 'satellite'` | Current base map style |
| `leftPanelOpen` | `boolean` | Left Detail Panel visibility |
| `tableOpen` | `boolean` | Opportunities Table visibility |
| `rightDrawerOpen` | `boolean` | Right Drawer visibility |
| `selectedOpportunityId` | `string \| null` | Row selected in table |
| `selectedMapFeature` | `GeoJSON.Feature \| null` | Feature clicked on map |

### ViewState (react-map-gl)

Managed via `useMap` hook. Debounced 300ms before use as a TanStack Query key to prevent request floods (see AGENTS.md performance rules).

| Field | Type |
|---|---|
| `longitude` | `number` |
| `latitude` | `number` |
| `zoom` | `number` |
| `pitch` | `number` |
| `bearing` | `number` |
