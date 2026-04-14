# Mapbox GL JS API Patterns

Detailed API patterns and advanced usage for Mapbox GL JS v3.

## Map Constructor Options

```typescript
import mapboxgl from 'mapbox-gl';

mapboxgl.accessToken = process.env.NEXT_PUBLIC_MAPBOX_TOKEN!;

const map = new mapboxgl.Map({
  container: 'map',
  style: 'mapbox://styles/mapbox/standard',
  center: [139.6917, 35.6895],         // [lng, lat]
  zoom: 11,
  pitch: 45,                            // tilt (0-85°)
  bearing: -17.6,                       // rotation
  projection: 'mercator',               // or 'globe'
  maxZoom: 20,
  minZoom: 5,
  maxPitch: 85,
  antialias: true,
  hash: false,
  attributionControl: true,
  cooperativeGestures: false,
});
```

## Camera Animations

```typescript
// Smooth fly to location
map.flyTo({
  center: [139.7671, 35.6812],
  zoom: 15,
  pitch: 60,
  bearing: 30,
  duration: 2000,
  essential: true,
});

// Instant jump
map.jumpTo({ center: [139.7671, 35.6812], zoom: 15 });

// Smooth ease
map.easeTo({
  center: [139.7671, 35.6812],
  zoom: 14,
  duration: 1000,
});

// Fit bounds to data extent
map.fitBounds(
  [[139.5, 35.5], [140.0, 35.9]],
  { padding: 50, maxZoom: 15 }
);
```

## Event System

### Map Events

```typescript
map.on('load', () => {});           // all resources loaded
map.on('style.load', () => {});     // style loaded (add layers here)
map.on('moveend', () => {});        // camera movement finished
map.on('zoomend', () => {});        // zoom change finished
map.on('idle', () => {});           // map fully rendered and idle
map.on('error', (e) => {});         // error occurred
map.on('render', () => {});         // every render frame
```

### Layer Interaction Events

```typescript
map.on('click', 'layer-id', (e) => {
  const feature = e.features?.[0];
  if (!feature) return;
  const coords = e.lngLat;
  const props = feature.properties;
});

map.on('mouseenter', 'layer-id', () => {
  map.getCanvas().style.cursor = 'pointer';
});

map.on('mouseleave', 'layer-id', () => {
  map.getCanvas().style.cursor = '';
});
```

## Querying Features

```typescript
// Query rendered features at a point
const features = map.queryRenderedFeatures(point, {
  layers: ['transaction-circles'],
});

// Query rendered features in a bbox
const bbox: [mapboxgl.PointLike, mapboxgl.PointLike] = [
  [e.point.x - 5, e.point.y - 5],
  [e.point.x + 5, e.point.y + 5],
];
const selected = map.queryRenderedFeatures(bbox, {
  layers: ['zoning-fill'],
});

// Query source features (all, not just visible)
const allFeatures = map.querySourceFeatures('transactions', {
  sourceLayer: 'points',
  filter: ['>', 'pricePerSqm', 500000],
});
```

## Dynamic Source Data Update

```typescript
const source = map.getSource('transactions') as mapboxgl.GeoJSONSource;
if (source) {
  source.setData(newFeatureCollection);
}
```

## Filter Expressions

```typescript
// Filter by property
map.setFilter('transactions', ['==', 'useType', 'residential']);

// Range filter
map.setFilter('transactions', [
  'all',
  ['>=', 'pricePerSqm', 200000],
  ['<=', 'pricePerSqm', 800000],
]);

// Multiple conditions
map.setFilter('transactions', [
  'all',
  ['==', 'useType', 'residential'],
  ['>=', 'year', 2020],
  ['has', 'pricePerSqm'],
]);

// Clear filter
map.setFilter('transactions', null);
```

## Markers and Popups

```typescript
// Custom HTML marker
const el = document.createElement('div');
el.className = 'custom-marker';

new mapboxgl.Marker(el)
  .setLngLat([139.7671, 35.6812])
  .setPopup(
    new mapboxgl.Popup({ offset: 25 })
      .setHTML('<h3>Shibuya</h3><p>Price: ¥500,000/m²</p>')
  )
  .addTo(map);
```

## Controls

```typescript
map.addControl(new mapboxgl.NavigationControl(), 'top-right');
map.addControl(new mapboxgl.ScaleControl({ unit: 'metric' }), 'bottom-left');
map.addControl(new mapboxgl.FullscreenControl(), 'top-right');
map.addControl(
  new mapboxgl.GeolocateControl({
    positionOptions: { enableHighAccuracy: true },
    trackUserLocation: true,
  }),
  'top-right'
);
```

## Globe Projection with Atmosphere

```typescript
const map = new mapboxgl.Map({
  projection: 'globe',
  style: 'mapbox://styles/mapbox/standard',
});

map.on('style.load', () => {
  map.setFog({
    color: 'rgb(186, 210, 235)',
    'high-color': 'rgb(36, 92, 223)',
    'horizon-blend': 0.02,
    'space-color': 'rgb(11, 11, 25)',
    'star-intensity': 0.6,
  });
});
```

## Terrain

```typescript
map.on('style.load', () => {
  map.addSource('mapbox-dem', {
    type: 'raster-dem',
    url: 'mapbox://mapbox.mapbox-terrain-dem-v1',
    tileSize: 512,
    maxzoom: 14,
  });
  map.setTerrain({ source: 'mapbox-dem', exaggeration: 1.5 });

  map.addLayer({
    id: 'sky',
    type: 'sky',
    paint: {
      'sky-type': 'atmosphere',
      'sky-atmosphere-sun': [0.0, 0.0],
      'sky-atmosphere-sun-intensity': 15,
    },
  });
});
```

## Clustering

```typescript
map.addSource('transactions', {
  type: 'geojson',
  data: transactionGeoJSON,
  cluster: true,
  clusterMaxZoom: 14,
  clusterRadius: 50,
  clusterProperties: {
    sum: ['+', ['get', 'pricePerSqm']],
    count: ['+', 1],
  },
});

// Cluster circles
map.addLayer({
  id: 'clusters',
  type: 'circle',
  source: 'transactions',
  filter: ['has', 'point_count'],
  paint: {
    'circle-color': [
      'step', ['get', 'point_count'],
      '#51bbd6', 10,
      '#f1f075', 30,
      '#f28cb1',
    ],
    'circle-radius': [
      'step', ['get', 'point_count'],
      15, 10,
      20, 30,
      25,
    ],
  },
});

// Cluster count labels
map.addLayer({
  id: 'cluster-count',
  type: 'symbol',
  source: 'transactions',
  filter: ['has', 'point_count'],
  layout: {
    'text-field': '{point_count_abbreviated}',
    'text-size': 12,
  },
});

// Unclustered points
map.addLayer({
  id: 'unclustered-point',
  type: 'circle',
  source: 'transactions',
  filter: ['!', ['has', 'point_count']],
  paint: {
    'circle-color': '#11b4da',
    'circle-radius': 4,
  },
});

// Click cluster to zoom in
map.on('click', 'clusters', (e) => {
  const features = map.queryRenderedFeatures(e.point, { layers: ['clusters'] });
  const clusterId = features[0]?.properties?.cluster_id;
  const source = map.getSource('transactions') as mapboxgl.GeoJSONSource;
  source.getClusterExpansionZoom(clusterId, (err, zoom) => {
    if (err || zoom == null) return;
    map.easeTo({
      center: (features[0].geometry as GeoJSON.Point).coordinates as [number, number],
      zoom,
    });
  });
});
```

## Image Loading for Symbols

```typescript
map.on('style.load', () => {
  map.loadImage('/icons/school.png', (error, image) => {
    if (error) throw error;
    if (!image) return;
    map.addImage('school-icon', image);
  });
});
```
