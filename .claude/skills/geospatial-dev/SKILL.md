---
name: geospatial-dev
description: "MapLibre GL JS + PostGIS integration patterns for real estate data visualization. Use when working on map layers, spatial queries, GeoJSON data pipelines, or 3D extrusion effects."
metadata:
  version: "1.0.0"
  filePattern:
    - "src/components/map/**"
    - "src/features/*/components/Map*"
    - "**/*maplibre*"
    - "**/*geojson*"
    - "**/*spatial*"
---

# Geospatial Development Guide

MapLibre GL JS + PostGIS patterns for real estate investment data visualization.

## MapLibre Layer Types in This Project

| Layer | Type | Data | Purpose |
|-------|------|------|---------|
| Transaction Heatmap | `heatmap` | Point | Price density visualization |
| Transaction Circles | `circle` | Point | Individual transactions with price coloring |
| Zoning Polygons | `fill` | Polygon | Land use zone overlay |
| Disaster Risk | `fill-extrusion` | Polygon | 3D risk level visualization |
| Facility Markers | `symbol` | Point | Schools, hospitals, stations |
| Transit Lines | `line` | LineString | Railway/subway routes |

## MapLibre + react-map-gl Pattern

```typescript
import Map, { Source, Layer, type MapLayerMouseEvent } from 'react-map-gl/maplibre';

function PropertyMap() {
  const handleClick = (e: MapLayerMouseEvent) => {
    const feature = e.features?.[0];
    if (feature) {
      selectProperty(feature.properties as Property);
    }
  };

  return (
    <Map
      mapStyle="https://basemaps.cartocdn.com/gl/dark-matter-gl-style/style.json"
      interactiveLayerIds={['transaction-circles']}
      onClick={handleClick}
    >
      <Source id="transactions" type="geojson" data={transactionGeoJSON}>
        <Layer
          id="transaction-circles"
          type="circle"
          paint={{
            'circle-radius': ['interpolate', ['linear'], ['zoom'], 10, 3, 15, 8],
            'circle-color': [
              'interpolate', ['linear'], ['get', 'pricePerSqm'],
              100000, '#2196F3',
              500000, '#FF9800',
              1000000, '#F44336',
            ],
            'circle-opacity': 0.8,
          }}
        />
      </Source>
    </Map>
  );
}
```

## 3D Fill-Extrusion (Disaster Risk)

```typescript
<Layer
  id="disaster-risk-3d"
  type="fill-extrusion"
  paint={{
    'fill-extrusion-color': [
      'interpolate', ['linear'], ['get', 'riskScore'],
      0, '#4CAF50',
      50, '#FF9800',
      100, '#F44336',
    ],
    'fill-extrusion-height': ['*', ['get', 'riskScore'], 100],
    'fill-extrusion-opacity': 0.7,
  }}
/>
```

## PostGIS Spatial Queries

### Bounding Box Query (for viewport)
```sql
SELECT ST_AsGeoJSON(location)::jsonb AS geometry, price_per_sqm, area_sqm
FROM transactions
WHERE ST_Within(location, ST_MakeEnvelope($1, $2, $3, $4, 4326))
LIMIT 1000;
```

### Radius Query (for nearby search)
```sql
SELECT *, ST_Distance(
  location::geography,
  ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography
) AS distance_m
FROM facilities
WHERE ST_DWithin(
  location::geography,
  ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography,
  $3  -- radius in meters
)
ORDER BY distance_m;
```

### Feature Collection Builder
```sql
SELECT jsonb_build_object(
  'type', 'FeatureCollection',
  'features', COALESCE(jsonb_agg(
    jsonb_build_object(
      'type', 'Feature',
      'geometry', ST_AsGeoJSON(location)::jsonb,
      'properties', jsonb_build_object(
        'id', id,
        'pricePerSqm', price_per_sqm,
        'areaType', area_type
      )
    )
  ), '[]'::jsonb)
) AS geojson;
```

## GeoJSON RFC 7946 Compliance

- Coordinates MUST be [longitude, latitude] (not lat/lng)
- Use `geometry(Point, 4326)` for WGS84 coordinate system
- Feature `properties` must be a JSON object (not null for MapLibre)
- Validate GeoJSON at API boundary with Zod schema

## Performance Tips

- Limit features per layer to ~5000 for smooth rendering
- Use viewport-based queries (bounding box) to reduce data
- Cluster points at low zoom levels (`cluster: true` on GeoJSON source)
- Use `fill-extrusion` sparingly — high GPU cost
- Prefer vector tiles for datasets > 10,000 features
