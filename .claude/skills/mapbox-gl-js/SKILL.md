---
name: mapbox-gl-js
description: "Mapbox GL JS v3 development patterns for real estate data visualization. Use when working with Mapbox maps, Standard Style configuration, layer slots, 3D lighting, expressions, react-map-gl/mapbox integration, or migrating from MapLibre GL JS."
metadata:
  version: "1.0.0"
  filePattern:
    - "src/components/map/**"
    - "src/features/*/components/Map*"
    - "**/*mapbox*"
    - "**/*map-gl*"
---

# Mapbox GL JS Development Guide

Mapbox GL JS v3 patterns for the Terrasight real estate investment platform.

Load `references/api-patterns.md` and `references/react-integration.md` for
detailed code examples beyond what this summary covers.

## Version and License

- **Current**: Mapbox GL JS v3.20.0 (proprietary license, requires access token)
- **Rendering**: WebGL 2 mandatory (all modern browsers supported)
- **TypeScript**: First-class types since v3.5.0 (remove `@types/mapbox-gl`)
- **react-map-gl**: v8.0+ uses `react-map-gl/mapbox` endpoint for Mapbox GL JS

## MapLibre vs Mapbox: When to Switch

This project currently uses MapLibre GL JS via `react-map-gl/maplibre`.
Switching to Mapbox GL JS provides:

| Feature | MapLibre | Mapbox GL JS v3 |
|---------|----------|-----------------|
| License | MIT (free) | Proprietary (pay per map load) |
| 3D lighting presets | Manual | Built-in (Day/Dusk/Dawn/Night) |
| Standard Style | N/A | Auto-updating basemap |
| Layer slots | N/A | `bottom` / `middle` / `top` |
| 3D landmarks | N/A | Built-in |
| Globe projection | Community | Native with fog/atmosphere |
| Terrain shadows | N/A | Built-in |

**Decision**: Use MapLibre for cost-free development; switch to Mapbox when
3D lighting, Standard Style, or premium tile quality justifies the cost.

## Quick Start

### Installation

```bash
pnpm add mapbox-gl
# react-map-gl v8+ required for Mapbox GL JS v3.5+
pnpm add react-map-gl
```

### Access Token

```typescript
// .env.local
NEXT_PUBLIC_MAPBOX_TOKEN=pk.xxx

// lib/mapbox.ts
import mapboxgl from 'mapbox-gl';
mapboxgl.accessToken = process.env.NEXT_PUBLIC_MAPBOX_TOKEN!;
```

### Basic Map with react-map-gl

```typescript
import Map from 'react-map-gl/mapbox';
import 'mapbox-gl/dist/mapbox-gl.css';

function PropertyMap() {
  return (
    <Map
      mapboxAccessToken={process.env.NEXT_PUBLIC_MAPBOX_TOKEN}
      initialViewState={{
        longitude: 139.6917,
        latitude: 35.6895,
        zoom: 11,
        pitch: 45,
      }}
      mapStyle="mapbox://styles/mapbox/standard"
    />
  );
}
```

## Standard Style Configuration

The Standard Style auto-updates with no migration required. Configure via
`setConfigProperty` after `style.load`:

```typescript
import { useMap } from 'react-map-gl/mapbox';

function MapControls() {
  const { current: map } = useMap();

  const setLightPreset = (preset: 'day' | 'dusk' | 'dawn' | 'night') => {
    map?.getMap().setConfigProperty('basemap', 'lightPreset', preset);
  };

  const togglePOI = (show: boolean) => {
    map?.getMap().setConfigProperty('basemap', 'showPointOfInterestLabels', show);
  };

  const setTheme = (theme: 'default' | 'faded' | 'monochrome') => {
    map?.getMap().setConfigProperty('basemap', 'theme', theme);
  };
}
```

### Standard Style Config Properties

| Property | Type | Description |
|----------|------|-------------|
| `showPlaceLabels` | `bool` | Place label layers |
| `showRoadLabels` | `bool` | Road labels + shields |
| `showPointOfInterestLabels` | `bool` | POI icons and text |
| `showTransitLabels` | `bool` | Transit icons and text |
| `show3dObjects` | `bool` | 3D buildings, landmarks, trees, shadows |
| `theme` | `string` | `default` / `faded` / `monochrome` |
| `lightPreset` | `string` | `day` / `dusk` / `dawn` / `night` |
| `font` | `string` | Font family from predefined options |

## Layer Slots (Standard Style Only)

Custom layers insert into predetermined positions in the Standard basemap:

| Slot | Position |
|------|----------|
| `bottom` | Above polygons (land, water) |
| `middle` | Above roads, behind 3D buildings |
| `top` | Above POI labels, behind place/transit labels |
| (none) | Above all existing layers |

```typescript
map.addLayer({
  id: 'transaction-circles',
  type: 'circle',
  slot: 'middle',
  source: 'transactions',
  paint: {
    'circle-radius': ['interpolate', ['linear'], ['zoom'], 10, 3, 15, 8],
    'circle-color': [
      'interpolate', ['linear'], ['get', 'pricePerSqm'],
      100000, '#2196F3',
      500000, '#FF9800',
      1000000, '#F44336',
    ],
    'circle-emissive-strength': 0.8,
  },
});
```

## Source Types

### GeoJSON (primary for this project)

```typescript
map.addSource('transactions', {
  type: 'geojson',
  data: featureCollection,
  cluster: true,
  clusterMaxZoom: 14,
  clusterRadius: 50,
});
```

### Vector Tiles (for large datasets)

```typescript
map.addSource('zoning', {
  type: 'vector',
  url: 'mapbox://username.tileset-id',
});

map.addLayer({
  id: 'zoning-fill',
  type: 'fill',
  source: 'zoning',
  'source-layer': 'zoning_areas',
  paint: { 'fill-color': '#088', 'fill-opacity': 0.5 },
});
```

### Raster DEM (terrain)

```typescript
map.addSource('terrain-dem', {
  type: 'raster-dem',
  url: 'mapbox://mapbox.mapbox-terrain-dem-v1',
});
map.setTerrain({ source: 'terrain-dem', exaggeration: 1.5 });
```

## Layer Types for Real Estate

### Circle (transactions)

```typescript
{
  id: 'transactions',
  type: 'circle',
  slot: 'middle',
  source: 'transactions',
  paint: {
    'circle-radius': ['interpolate', ['linear'], ['zoom'], 10, 2, 16, 10],
    'circle-color': ['match', ['get', 'useType'],
      'residential', '#4CAF50',
      'commercial', '#2196F3',
      'industrial', '#FF9800',
      '#999',
    ],
    'circle-opacity': 0.8,
    'circle-emissive-strength': 1,
  },
}
```

### Fill Extrusion (3D buildings / risk zones)

```typescript
{
  id: 'disaster-risk-3d',
  type: 'fill-extrusion',
  slot: 'middle',
  source: 'risk-zones',
  paint: {
    'fill-extrusion-color': [
      'interpolate', ['linear'], ['get', 'riskScore'],
      0, '#4CAF50',
      50, '#FF9800',
      100, '#F44336',
    ],
    'fill-extrusion-height': ['*', ['get', 'riskScore'], 100],
    'fill-extrusion-opacity': 0.7,
    'fill-extrusion-ambient-occlusion-ground-radius': 5,
    'fill-extrusion-flood-light-intensity': 0.3,
  },
}
```

### Heatmap (price density)

```typescript
{
  id: 'price-heatmap',
  type: 'heatmap',
  slot: 'bottom',
  source: 'transactions',
  paint: {
    'heatmap-weight': ['interpolate', ['linear'], ['get', 'pricePerSqm'],
      0, 0, 1000000, 1,
    ],
    'heatmap-intensity': ['interpolate', ['linear'], ['zoom'], 0, 1, 15, 3],
    'heatmap-radius': ['interpolate', ['linear'], ['zoom'], 0, 2, 15, 20],
    'heatmap-color': [
      'interpolate', ['linear'], ['heatmap-density'],
      0, 'rgba(33,102,172,0)',
      0.2, 'rgb(103,169,207)',
      0.4, 'rgb(209,229,240)',
      0.6, 'rgb(253,219,199)',
      0.8, 'rgb(239,138,98)',
      1, 'rgb(178,24,43)',
    ],
  },
}
```

### Symbol (facility markers)

```typescript
{
  id: 'facilities',
  type: 'symbol',
  slot: 'top',
  source: 'facilities',
  layout: {
    'icon-image': ['get', 'icon'],
    'icon-size': 0.8,
    'text-field': ['get', 'name'],
    'text-size': 11,
    'text-offset': [0, 1.2],
    'text-anchor': 'top',
  },
  paint: {
    'text-color': '#333',
    'text-halo-color': '#fff',
    'text-halo-width': 1,
    'text-emissive-strength': 1,
    'icon-emissive-strength': 1,
  },
}
```

## Expressions Reference

### Data-driven styling

```typescript
// Match (categorical)
['match', ['get', 'category'], 'A', '#f00', 'B', '#0f0', '#999']

// Interpolate (continuous)
['interpolate', ['linear'], ['get', 'value'], 0, '#blue', 100, '#red']

// Step (discrete)
['step', ['get', 'value'], '#green', 50, '#yellow', 80, '#red']

// Case (conditional)
['case',
  ['<', ['get', 'price'], 100000], '#4CAF50',
  ['<', ['get', 'price'], 500000], '#FF9800',
  '#F44336',
]
```

### Zoom-driven styling

```typescript
['interpolate', ['exponential', 1.5], ['zoom'],
  10, 2,   // zoom 10 → radius 2
  15, 12,  // zoom 15 → radius 12
  20, 30,  // zoom 20 → radius 30
]
```

### v3 New Expressions

```typescript
// Random value per feature
['random']

// HSL color
['hsl', 200, 80, 50]

// Distance from geometry (meters)
['distance', { type: 'Point', coordinates: [139.69, 35.68] }]

// Config value (Standard Style)
['config', 'lightPreset']
```

## 3D Lighting API

```typescript
map.setLights([
  {
    id: 'ambient',
    type: 'ambient',
    properties: { color: 'white', intensity: 0.4 },
  },
  {
    id: 'directional',
    type: 'directional',
    properties: {
      color: 'white',
      intensity: 0.8,
      direction: [200, 40],
      'cast-shadows': true,
      'shadow-intensity': 0.3,
    },
  },
]);
```

## react-map-gl v8 Integration

### Import Path (Mapbox vs MapLibre)

```typescript
// Mapbox GL JS v3.5+
import Map, { Source, Layer, Marker, Popup } from 'react-map-gl/mapbox';
import 'mapbox-gl/dist/mapbox-gl.css';

// MapLibre (current project setup)
import Map, { Source, Layer, Marker, Popup } from 'react-map-gl/maplibre';
import 'maplibre-gl/dist/maplibre-gl.css';
```

### Event Handling

```typescript
import Map, { type MapLayerMouseEvent } from 'react-map-gl/mapbox';

function PropertyMap() {
  const handleClick = useCallback((e: MapLayerMouseEvent) => {
    const feature = e.features?.[0];
    if (!feature) return;
    setSelectedProperty(feature.properties as TransactionProperty);
  }, []);

  const handleMouseEnter = useCallback(() => {
    setCursor('pointer');
  }, []);

  return (
    <Map
      mapboxAccessToken={token}
      interactiveLayerIds={['transaction-circles']}
      onClick={handleClick}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={() => setCursor('')}
      cursor={cursor}
    />
  );
}
```

### Viewport Sync with Zustand

```typescript
import { useMapStore } from '@/stores/mapStore';

function SyncedMap() {
  const viewState = useMapStore((s) => s.viewState);
  const setViewState = useMapStore((s) => s.setViewState);

  return (
    <Map
      {...viewState}
      onMove={(e) => setViewState(e.viewState)}
      mapboxAccessToken={token}
      mapStyle="mapbox://styles/mapbox/standard"
    />
  );
}
```

## Migration: MapLibre → Mapbox

### Step 1: Dependencies

```bash
pnpm remove maplibre-gl
pnpm add mapbox-gl
```

### Step 2: Import paths

```typescript
// Before
import Map from 'react-map-gl/maplibre';
import 'maplibre-gl/dist/maplibre-gl.css';

// After
import Map from 'react-map-gl/mapbox';
import 'mapbox-gl/dist/mapbox-gl.css';
```

### Step 3: Map style

```typescript
// Before (MapLibre — free tile provider)
mapStyle="https://basemaps.cartocdn.com/gl/dark-matter-gl-style/style.json"

// After (Mapbox Standard — auto-updating)
mapStyle="mapbox://styles/mapbox/standard"
```

### Step 4: Access token

```typescript
<Map mapboxAccessToken={process.env.NEXT_PUBLIC_MAPBOX_TOKEN} />
```

### Step 5: Leverage v3 features

- Add `slot` property to custom layers
- Configure Standard Style with `setConfigProperty`
- Use `*-emissive-strength` paint properties for lighting

## Performance Rules

- GeoJSON sources: limit to ~5,000 features per layer
- Use `cluster: true` for point data at low zoom levels
- Use vector tiles (`mapbox://`) for datasets > 10,000 features
- `fill-extrusion` is GPU-intensive — limit to essential layers
- Prefer `style.load` event over `load` for adding layers
- Debounce viewport-based data fetching (Zustand → TanStack Query)

## Pricing Awareness

- Mapbox bills per **map load** (every `new Map()` instantiation)
- Free tier: 50,000 map loads/month (web)
- Tile requests included in map load pricing
- Monitor usage at [account.mapbox.com](https://account.mapbox.com)

## GeoJSON Compliance (RFC 7946)

- Coordinates: `[longitude, latitude]` order always
- Feature `properties`: must be a JSON object (not null)
- Use `geometry(Point, 4326)` for WGS84 in PostGIS
- Validate at API boundary with Zod schema
