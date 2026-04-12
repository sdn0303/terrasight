# State Management

## Contents

- [Zustand Store Pattern](#zustand-store-pattern)
- [State and Action Separation](#state-and-action-separation)
- [Selector Discipline](#selector-discipline)
- [Subscribe Outside React](#subscribe-outside-react)
- [URL State with nuqs](#url-state-with-nuqs)
- [Anti-patterns](#anti-patterns)

---

## Zustand Store Pattern

Always use `subscribeWithSelector` + `devtools` + `persist`:

```typescript
import { create } from 'zustand';
import { devtools, persist, subscribeWithSelector } from 'zustand/middleware';

export const useMapStore = create<MapStore>()(
  devtools(
    subscribeWithSelector(
      persist(
        (set, get) => ({
          // state + actions
        }),
        {
          name: 'map-store',
          partialize: (state) => ({ visibleLayers: state.visibleLayers }),
        },
      ),
    ),
  ),
);
```

## State and Action Separation

```typescript
interface MapState {
  visibleLayers: Set<string>;
  selectedProperty: Property | null;
  viewState: ViewState;
}

interface MapActions {
  toggleLayer: (layerId: string) => void;
  selectProperty: (property: Property | null) => void;
  updateViewState: (viewState: ViewState) => void;
}

type MapStore = MapState & MapActions;
```

## Selector Discipline

```typescript
// Good — only re-renders when visibleLayers changes
const visibleLayers = useMapStore((state) => state.visibleLayers);

// Bad — re-renders on any state change
const { visibleLayers, selectedProperty } = useMapStore();
```

## Subscribe Outside React

```typescript
useMapStore.subscribe(
  (state) => state.selectedProperty,
  (selected) => {
    console.log('Selected:', selected?.id);
  },
);
```

## URL State with nuqs

Use `nuqs` for filters and pagination that should survive refresh:

```typescript
const [areaCode, setAreaCode] = useQueryState('area');
const [page, setPage] = useQueryState('page', parseAsInteger.withDefault(1));
```

## Anti-patterns

- Destructuring full store without selectors (triggers re-render on any change)
- Deriving TanStack Query keys from high-frequency Zustand state without debounce
- Mixing server state (API data) in Zustand — use TanStack Query for server state
- Multiple stores for the same domain concern

## Project Convention

- Store files in `src/stores/`
- One store per domain (map, filters, auth)
- Export from `src/stores/index.ts`
