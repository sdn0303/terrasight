# react-map-gl v8 + Mapbox GL JS Integration

Patterns for using react-map-gl v8 with Mapbox GL JS v3 in Next.js App Router.

## Installation

```bash
pnpm add mapbox-gl react-map-gl
```

## Import Convention

```typescript
// Mapbox GL JS v3.5+
import Map, {
  Source,
  Layer,
  Marker,
  Popup,
  NavigationControl,
  ScaleControl,
  GeolocateControl,
  FullscreenControl,
  useMap,
  type MapLayerMouseEvent,
  type ViewStateChangeEvent,
  type MapRef,
} from 'react-map-gl/mapbox';
import 'mapbox-gl/dist/mapbox-gl.css';

// NOT from 'react-map-gl' directly
// NOT from 'react-map-gl/maplibre' (that's for MapLibre)
```

## Complete Map Component

```typescript
'use client';

import { useCallback, useRef, useState } from 'react';
import Map, {
  Source,
  Layer,
  Popup,
  NavigationControl,
  type MapLayerMouseEvent,
  type MapRef,
} from 'react-map-gl/mapbox';
import 'mapbox-gl/dist/mapbox-gl.css';

import type { TransactionFeature } from '@/features/transactions/types';
import { useMapStore } from '@/stores/mapStore';

const MAPBOX_TOKEN = process.env.NEXT_PUBLIC_MAPBOX_TOKEN!;

export function PropertyMap() {
  const mapRef = useRef<MapRef>(null);
  const viewState = useMapStore((s) => s.viewState);
  const setViewState = useMapStore((s) => s.setViewState);
  const [selectedFeature, setSelectedFeature] = useState<TransactionFeature | null>(null);

  const handleClick = useCallback((e: MapLayerMouseEvent) => {
    const feature = e.features?.[0];
    if (!feature) {
      setSelectedFeature(null);
      return;
    }
    setSelectedFeature(feature as unknown as TransactionFeature);
  }, []);

  const handleStyleLoad = useCallback(() => {
    const map = mapRef.current?.getMap();
    if (!map) return;
    map.setConfigProperty('basemap', 'lightPreset', 'day');
    map.setConfigProperty('basemap', 'showPointOfInterestLabels', false);
  }, []);

  return (
    <Map
      ref={mapRef}
      {...viewState}
      onMove={(e) => setViewState(e.viewState)}
      mapboxAccessToken={MAPBOX_TOKEN}
      mapStyle="mapbox://styles/mapbox/standard"
      interactiveLayerIds={['transaction-circles']}
      onClick={handleClick}
      onStyleLoad={handleStyleLoad}
      maxPitch={85}
      reuseMaps
    >
      <NavigationControl position="top-right" />

      <Source id="transactions" type="geojson" data={transactionGeoJSON}>
        <Layer
          id="transaction-circles"
          type="circle"
          slot="middle"
          paint={{
            'circle-radius': ['interpolate', ['linear'], ['zoom'], 10, 3, 15, 8],
            'circle-color': [
              'interpolate', ['linear'], ['get', 'pricePerSqm'],
              100000, '#2196F3',
              500000, '#FF9800',
              1000000, '#F44336',
            ],
            'circle-opacity': 0.8,
            'circle-emissive-strength': 1,
          }}
        />
      </Source>

      {selectedFeature && (
        <Popup
          longitude={selectedFeature.geometry.coordinates[0]}
          latitude={selectedFeature.geometry.coordinates[1]}
          anchor="bottom"
          onClose={() => setSelectedFeature(null)}
        >
          <PropertyPopupContent feature={selectedFeature} />
        </Popup>
      )}
    </Map>
  );
}
```

## Dynamic Layer Visibility

```typescript
function LayerToggle({ layerId, label }: { layerId: string; label: string }) {
  const { current: map } = useMap();
  const [visible, setVisible] = useState(true);

  const toggle = () => {
    const newVisibility = visible ? 'none' : 'visible';
    map?.getMap().setLayoutProperty(layerId, 'visibility', newVisibility);
    setVisible(!visible);
  };

  return (
    <button onClick={toggle}>
      {visible ? '👁' : '🚫'} {label}
    </button>
  );
}
```

## Viewport-based Data Fetching

```typescript
import { useQuery } from '@tanstack/react-query';
import { useDebouncedValue } from '@/hooks/useDebouncedValue';

function useViewportTransactions() {
  const bounds = useMapStore((s) => s.bounds);
  const debouncedBounds = useDebouncedValue(bounds, 300);

  return useQuery({
    queryKey: ['transactions', 'viewport', debouncedBounds],
    queryFn: () => fetchTransactionsByBounds(debouncedBounds!),
    enabled: !!debouncedBounds,
    staleTime: 30_000,
  });
}
```

## Multiple Layers from One Source

```typescript
<Source id="transactions" type="geojson" data={data} cluster clusterRadius={50}>
  {/* Cluster circles */}
  <Layer
    id="clusters"
    type="circle"
    slot="middle"
    filter={['has', 'point_count']}
    paint={{
      'circle-color': ['step', ['get', 'point_count'], '#51bbd6', 10, '#f28cb1'],
      'circle-radius': ['step', ['get', 'point_count'], 15, 10, 25],
    }}
  />

  {/* Cluster count */}
  <Layer
    id="cluster-count"
    type="symbol"
    slot="top"
    filter={['has', 'point_count']}
    layout={{ 'text-field': '{point_count_abbreviated}', 'text-size': 12 }}
  />

  {/* Unclustered points */}
  <Layer
    id="unclustered"
    type="circle"
    slot="middle"
    filter={['!', ['has', 'point_count']]}
    paint={{ 'circle-color': '#11b4da', 'circle-radius': 6 }}
  />
</Source>
```

## Map Style Switching

```typescript
const STYLES = {
  standard: 'mapbox://styles/mapbox/standard',
  satellite: 'mapbox://styles/mapbox/standard-satellite',
  streets: 'mapbox://styles/mapbox/streets-v12',
  dark: 'mapbox://styles/mapbox/dark-v11',
  light: 'mapbox://styles/mapbox/light-v11',
} as const;

function StyleSwitcher() {
  const [style, setStyle] = useState<keyof typeof STYLES>('standard');

  return (
    <Map
      mapStyle={STYLES[style]}
      mapboxAccessToken={MAPBOX_TOKEN}
    />
  );
}
```

## Next.js App Router: Dynamic Import

Mapbox GL JS uses `window` and WebGL. Use dynamic import for SSR compatibility:

```typescript
// components/map/MapContainer.tsx
'use client';

import dynamic from 'next/dynamic';

const PropertyMap = dynamic(
  () => import('./PropertyMap').then((mod) => mod.PropertyMap),
  { ssr: false, loading: () => <MapSkeleton /> }
);

export function MapContainer() {
  return <PropertyMap />;
}
```

## CSS Import in Layout

```typescript
// app/layout.tsx or a client component
import 'mapbox-gl/dist/mapbox-gl.css';
```

## Zustand Map Store

```typescript
import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import type { ViewState, LngLatBounds } from 'react-map-gl/mapbox';

interface MapState {
  viewState: Partial<ViewState>;
  bounds: LngLatBounds | null;
  selectedLayerIds: string[];
}

interface MapActions {
  setViewState: (vs: Partial<ViewState>) => void;
  setBounds: (bounds: LngLatBounds) => void;
  toggleLayer: (id: string) => void;
}

export const useMapStore = create<MapState & MapActions>()(
  subscribeWithSelector((set) => ({
    viewState: {
      longitude: 139.6917,
      latitude: 35.6895,
      zoom: 11,
      pitch: 45,
      bearing: 0,
    },
    bounds: null,
    selectedLayerIds: ['transaction-circles'],

    setViewState: (vs) => set({ viewState: vs }),
    setBounds: (bounds) => set({ bounds }),
    toggleLayer: (id) =>
      set((s) => ({
        selectedLayerIds: s.selectedLayerIds.includes(id)
          ? s.selectedLayerIds.filter((l) => l !== id)
          : [...s.selectedLayerIds, id],
      })),
  }))
);
```
