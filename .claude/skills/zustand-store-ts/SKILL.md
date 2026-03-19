---
name: zustand-store-ts
description: "Create Zustand stores with TypeScript, subscribeWithSelector middleware, and proper state/action separation. Use when building React state management or creating global stores."
metadata:
  version: "1.0.0"
  filePattern:
    - "src/stores/**"
    - "src/store/**"
---

# Zustand Store Patterns

## Always Use subscribeWithSelector

```typescript
import { create } from 'zustand';
import { devtools, persist, subscribeWithSelector } from 'zustand/middleware';

export const useMapStore = create<MapStore>()(
  devtools(
    subscribeWithSelector(
      persist(
        (set, get) => ({
          // state and actions
        }),
        {
          name: 'map-store',
          partialize: (state) => ({ visibleLayers: state.visibleLayers }),
        }
      )
    )
  )
);
```

## Separate State and Actions

```typescript
export interface MapState {
  visibleLayers: Set<string>;
  selectedProperty: Property | null;
  viewState: ViewState;
}

export interface MapActions {
  toggleLayer: (layerId: string) => void;
  selectProperty: (property: Property | null) => void;
  updateViewState: (viewState: ViewState) => void;
}

export type MapStore = MapState & MapActions;
```

## Use Individual Selectors (Prevent Unnecessary Re-renders)

```typescript
// Good — only re-renders when `visibleLayers` changes
const visibleLayers = useMapStore((state) => state.visibleLayers);

// Bad — re-renders on any state change
const { visibleLayers, selectedProperty } = useMapStore();
```

## Subscribe Outside React

```typescript
useMapStore.subscribe(
  (state) => state.selectedProperty,
  (selectedProperty) => {
    // React to property selection changes outside React
    console.log('Selected:', selectedProperty?.id);
  }
);
```

## Project Convention

- Store files go in `src/stores/`
- One store per domain concern (map, filters, auth)
- Export from `src/stores/index.ts`
- Tests in `src/stores/*.test.ts`
