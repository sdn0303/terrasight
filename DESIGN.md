# Urban Stratigraphy Design System

## Design Philosophy

Urban Stratigraphy treats the city as layered geological strata. Just as geologists read the earth through cross-sections of rock, soil, and sediment, this platform lets real estate investors read Tokyo through stacked data layers -- land prices, flood risk, soil composition, infrastructure, and zoning. The Japanese title "地層" (chisou, meaning "geological stratum") anchors this metaphor throughout the interface.

The visual language is dark, dense, and information-first. A near-black canvas recedes so that map data and color-coded layers dominate the visual hierarchy. Monospaced type for data, sans-serif for headings, and cyan accents on dark surfaces evoke a command-center aesthetic.

## Color Tokens

All color tokens are defined as CSS custom properties in `:root` scope.

Source of truth: `services/frontend/src/app/globals.css`

### Base Palette

| Token | Value | Usage |
|---|---|---|
| `--bg-primary` | `#0c0c14` | Page background, map canvas background |
| `--bg-secondary` | `#13131e` | Panels, cards, popover surfaces |
| `--bg-tertiary` | `#1a1a28` | Secondary/muted surface, hover states |
| `--text-primary` | `#e4e4e7` | Body text, data values |
| `--text-secondary` | `#a1a1aa` | Labels, descriptions |
| `--text-muted` | `#52525b` | Disabled text, category headers |
| `--text-heading` | `#f4f4f5` | Panel titles, headings |
| `--border-primary` | `rgba(63, 63, 70, 0.5)` | Panel borders, dividers, input borders |

### Accent Colors

| Token | Value | Usage |
|---|---|---|
| `--accent-cyan` | `#22d3ee` | Primary accent, active indicators, interactive highlights, ring/focus |
| `--accent-danger` | `#e04030` | Destructive actions, high-risk indicators |
| `--accent-warning` | `#ffd000` | Warnings, demo mode badge, mid-risk flood depth |
| `--accent-success` | `#10b981` | Success states |
| `--hover-accent` | `rgba(34, 211, 238, 0.08)` | Active layer row background, hover highlight |

### shadcn/ui Theme Mapping (Zinc Dark)

These tokens bridge the custom palette to shadcn/ui component internals:

| Token | Value | Maps to |
|---|---|---|
| `--background` | `#0c0c14` | Same as `--bg-primary` |
| `--foreground` | `#e4e4e7` | Same as `--text-primary` |
| `--card` / `--card-foreground` | `#13131e` / `#e4e4e7` | Card surfaces |
| `--popover` / `--popover-foreground` | `#13131e` / `#e4e4e7` | Tooltip/popover surfaces |
| `--primary` / `--primary-foreground` | `#22d3ee` / `#0c0c14` | Cyan-on-dark for primary buttons |
| `--secondary` / `--secondary-foreground` | `#1a1a28` / `#e4e4e7` | Tertiary surface buttons |
| `--muted` / `--muted-foreground` | `#1a1a28` / `#a1a1aa` | Muted backgrounds and text |
| `--accent` / `--accent-foreground` | `rgba(34, 211, 238, 0.08)` / `#e4e4e7` | Accent surface |
| `--destructive` | `#e04030` | Destructive variant |
| `--border` | `rgba(63, 63, 70, 0.5)` | All borders |
| `--input` | `rgba(63, 63, 70, 0.5)` | Input borders |
| `--ring` | `#22d3ee` | Focus ring |
| `--radius` | `0.5rem` | Base border-radius (sm: -4px, md: -2px, lg: base, xl: +4px) |

### Layer Color Tokens

Each data layer has a dedicated color token used for the layer indicator dot in the panel and the map paint expression.

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

## Typography

Source of truth: `services/frontend/src/app/layout.tsx`, `globals.css`

### Font Stack

| Token | Value | Usage |
|---|---|---|
| `--font-sans` | `"Geist Sans", system-ui, sans-serif` | Headings, layer names, body text |
| `--font-mono` | `"Geist Mono", monospace, system-ui` | Data values, coordinates, status bar, category labels, popup cards |

Geist Mono is loaded via `geist/font/mono` and applied as a CSS variable class on `<html>`. The sans-serif variant is set as the body default via `font-family: var(--font-sans)`.

### Type Scale (in use)

| Context | Size | Weight | Tracking | Font |
|---|---|---|---|---|
| Panel title ("地層") | `text-base` (16px) | `font-bold` | `0.05em` | Sans |
| Panel subtitle ("URBAN STRATIGRAPHY") | `10px` | Normal | `0.12em` | Mono |
| Category header (e.g., "投資価値") | `9px` | Normal | `0.15em` | Mono |
| Layer toggle label | `text-xs` (12px) | Normal | Default | Sans |
| Active count badge | `9px` | Normal | Default | Mono |
| Popup card header | `10px` | Normal | `0.1em` | Mono |
| Popup card data rows | `11px` | Normal | Default | Mono |
| Status bar | `10px` | Normal | Default | Mono |
| Loading screen | `13px` | Normal | `0.1em` | Mono |

## Component Patterns

### shadcn/ui Components

The project uses shadcn/ui (style: `new-york`) with the following components:

- **Button** (`button.tsx`) -- standard variants
- **Card** (`card.tsx`) -- `bg-card` / `text-card-foreground`, rounded-xl, shadow-sm
- **Collapsible** (`collapsible.tsx`) -- layer category expand/collapse
- **ScrollArea** (`scroll-area.tsx`) -- scrollable panels
- **Separator** (`separator.tsx`) -- dividers
- **Sheet** (`sheet.tsx`) -- mobile layer panel (slides from left)
- **Skeleton** (`skeleton.tsx`) -- loading placeholders
- **Toggle** (`toggle.tsx`) -- toggle buttons
- **Tooltip** (`tooltip.tsx`) -- wrapped at root via `TooltipProvider`

### Map Layer Component Pattern

Each of the 24 layers follows a consistent pattern defined in `services/frontend/src/components/map/layers/`.

**API layer** (receives live data from backend):
```
interface Props { data: FeatureCollection; visible: boolean }
```
Returns `null` when `!visible`. Otherwise renders a `<Source>` with `type="geojson"` wrapping one or more `<Layer>` elements with MapLibre paint expressions.

**Static layer** (loads GeoJSON from `/geojson/` at mount):
```
interface Props { visible: boolean }
```
The `<Source>` data prop points to a static file path like `"/geojson/admin-boundary-tokyo.geojson"`.

**Layer type examples**:
- `circle` -- point data (land prices, stations, schools, medical)
- `fill` -- polygon areas (DID, zoning, flood history)
- `fill-extrusion` -- 3D polygons (flood risk depth, buildings)
- `line` -- linear features (fault lines, admin boundaries, railways)
- `symbol` -- label layers (admin boundary names)

Some layers compose multiple sub-layers (e.g., AdminBoundaryLayer renders fill + line + symbol).

### PopupCard Pattern

A single, config-driven `PopupCard` component handles click-inspect for all layers. It reads `popupFields` from the `LayerConfig` in `layers.ts` and renders key-value rows. No per-layer popup templates exist.

- Header: layer name in Japanese, cyan text, separated by a bottom border
- Body: label (secondary color) + value (primary color) pairs, right-aligned values
- Monospaced font throughout at 11px
- Max width: 240px

### Layer Panel Pattern

The `LayerPanel` component uses a responsive strategy:

- **Desktop** (>= 1280px): fixed 280px sidebar on the left, animated with Framer Motion (slide in/out)
- **Mobile/Tablet**: shadcn/ui `Sheet` sliding from the left, triggered by a hamburger button

Content is shared via `LayerPanelContent`, which renders collapsible category groups with toggle buttons per layer. Each layer row shows a colored indicator dot (using the layer's CSS color token) and the Japanese layer name.

### StatusBar Pattern

A fixed 28px bar at the bottom of the viewport. Monospaced, 10px. Displays coordinates, zoom level, DEMO mode indicator (warning yellow), and loading state (cyan).

## Layer Categories

Source of truth: `services/frontend/src/lib/layers.ts`

### HOW MUCH? (投資価値) -- `value`

Layers for assessing investment value and market context.

| Layer ID | Name | Source | Default |
|---|---|---|---|
| `landprice` | Land Price (地価公示) | API | On |
| `flood_history` | Flood History (浸水履歴) | Static | Off |
| `did` | DID Area (人口集中地区) | Static | Off |
| `station` | Railway Stations (鉄道駅) | Static | Off |

### IS IT SAFE? (リスク評価) -- `risk`

Natural disaster and hazard assessment layers.

| Layer ID | Name | Source | Default |
|---|---|---|---|
| `flood` | Flood Risk (洪水浸水) | API | Off |
| `steep_slope` | Steep Slope (急傾斜地) | API | Off |
| `liquefaction` | Liquefaction Risk (液状化危険度) | Static | Off |
| `seismic` | Seismic Hazard (地震動・震源断層) | Static | Off |
| `fault` | Fault Lines (断層線) | Static | Off |
| `volcano` | Volcanoes (火山) | Static | Off |
| `landslide` | Landslide Risk (土砂災害) | Static | Off |
| `tsunami` | Tsunami Risk (津波浸水) | Static | Off |

### WHAT'S THE GROUND? (地盤) -- `ground`

Subsurface and terrain composition layers.

| Layer ID | Name | Source | Default |
|---|---|---|---|
| `landform` | Landform (地形分類) | Static | Off |
| `geology` | Geology (表層地質) | Static | Off |
| `soil` | Soil (土壌図) | Static | Off |

### WHAT'S NEARBY? (インフラ) -- `infra`

Surrounding infrastructure and amenity layers.

| Layer ID | Name | Source | Default |
|---|---|---|---|
| `schools` | Schools (学校) | API | Off |
| `medical` | Medical (医療機関) | API | Off |
| `school_district` | School Districts (小学校区) | Static | Off |
| `park` | Parks (都市公園) | Static | Off |
| `railway` | Railway Lines (鉄道路線) | Static | Off |

### WHERE AM I? (オリエンテーション) -- `orientation`

Spatial context and planning framework layers.

| Layer ID | Name | Source | Default |
|---|---|---|---|
| `admin_boundary` | Admin Boundary (市町村境界) | Static | On |
| `zoning` | Zoning (用途地域) | API | On |
| `population_mesh` | Population Mesh (将来人口メッシュ) | Static | Off |
| `urban_plan` | Urban Planning Zones (立地適正化) | Static | Off |

## Dark Mode / Theme

The application is dark-only. The `<html>` element carries the `dark` class permanently (set in `layout.tsx`). There is no light mode toggle.

All theming flows through CSS custom properties defined in `:root` in `globals.css`. The Tailwind CSS v4 `@theme inline` block maps these CSS variables to Tailwind's color utility system (e.g., `bg-background`, `text-foreground`, `bg-card`).

The border-radius scale derives from a single `--radius` token (0.5rem):
- `--radius-sm`: `calc(var(--radius) - 4px)` = ~4px
- `--radius-md`: `calc(var(--radius) - 2px)` = ~6px
- `--radius-lg`: `var(--radius)` = 8px
- `--radius-xl`: `calc(var(--radius) + 4px)` = ~12px

## Spacing and Layout

### Viewport Structure

The root layout is a full-viewport container (`h-screen w-screen overflow-hidden`) with all panels overlaid as fixed-position elements on top of the map canvas.

```
+-----+--------------------------------------------+
|     |                                            |
| L   |              MapGL (100% x 100%)           |
| a   |          pitch: 45, bearing: 0             |
| y   |         3D terrain + buildings             |
| e   |                                            |
| r   |      [PopupCard] (centered overlay)        |
|     |                                            |
| 280 |                        [ScoreCard]         |
| px  |                        [ComparePanel]      |
|     |                        [DashboardStats]    |
|     |                    [NavControl bottom-right]|
+-----+--------------------------------------------+
| StatusBar (28px, full width, z-20)               |
+--------------------------------------------------+
```

### Key Dimensions

| Element | Size |
|---|---|
| Layer panel (desktop) | 280px wide, full height minus status bar |
| Status bar | 28px tall, full width |
| Popup card | max-width 240px |
| Mobile menu button | 36px (w-9 h-9) |
| Layer indicator dot | 8px (w-2 h-2) |
| Layer row padding | `px-2 py-1.5` |
| Category section padding | `px-4 py-1` |
| Panel header padding | `px-4 pt-4 pb-2` |

### Map Configuration

| Property | Value |
|---|---|
| Base style | CARTO Dark Matter (`dark-matter-gl-style`) |
| Default center | `[139.767, 35.681]` (Tokyo) |
| Default zoom | 12 |
| Default pitch | 45 degrees |
| Terrain source | AWS Elevation Tiles (terrarium encoding) |
| Terrain exaggeration | 1.5x |
| 3D buildings | CARTO vector tiles, fill-extrusion, color `#1e1e2e`, opacity 0.7 |
| Move debounce | 300ms |

### Responsive Breakpoints

| Breakpoint | Behavior |
|---|---|
| < 1280px | Layer panel rendered as Sheet (slide-from-left), hamburger trigger |
| >= 1280px | Layer panel rendered as fixed 280px sidebar with Framer Motion animation |
