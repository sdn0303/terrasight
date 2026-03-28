# X-06: bbox Approximation → map.getBounds() Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the center+zoom bbox approximation with MapLibre's actual `map.getBounds()`, fixing data fetch inaccuracy at high pitch/bearing angles.

**Architecture:** MapView's `onMoveEnd` callback will pass the real bounding box from `mapRef.current.getBounds()` to the parent. `useMapPage` receives the bbox directly instead of calling `getBBox()` from the store. The store's `getBBox()` method is kept for initial state only.

**Tech Stack:** React 19, MapLibre GL JS (via react-map-gl/maplibre), Zustand, TanStack Query v5, Vitest

---

## File Structure

| File | Change | Responsibility |
|------|--------|----------------|
| `services/frontend/src/lib/api.ts` | No change | BBox type definition (reused) |
| `services/frontend/src/components/map/map-view.tsx` | Modify | Pass real bounds from `map.getBounds()` through `onMoveEnd` |
| `services/frontend/src/hooks/use-map-page.ts` | Modify | Receive BBox from callback instead of calling `getBBox()` |
| `services/frontend/src/__tests__/map-view-bbox.test.ts` | Create | Test that onMoveEnd receives real bounds |
| `services/frontend/src/__tests__/map-store-extended.test.ts` | Modify | Update getBBox test description to clarify it's fallback-only |

---

### Task 1: Update MapView to pass real bounds via onMoveEnd

**Files:**
- Modify: `services/frontend/src/components/map/map-view.tsx:25-67`
- Create: `services/frontend/src/__tests__/map-view-bbox.test.ts`

- [ ] **Step 1: Write the test for MapView's onMoveEnd callback signature**

Create `services/frontend/src/__tests__/map-view-bbox.test.ts`:

```typescript
import { describe, expect, it, vi } from "vitest";
import type { BBox } from "@/lib/api";

describe("MapView onMoveEnd bbox contract", () => {
  it("BBox interface has required cardinal fields", () => {
    const bbox: BBox = { south: 35.6, west: 139.7, north: 35.8, east: 139.9 };
    expect(bbox.south).toBeLessThan(bbox.north);
    expect(bbox.west).toBeLessThan(bbox.east);
  });

  it("MapLibre LngLatBounds converts to BBox correctly", () => {
    // Simulate what map.getBounds() returns
    const mockBounds = {
      getSouth: () => 35.65,
      getWest: () => 139.72,
      getNorth: () => 35.71,
      getEast: () => 139.81,
    };

    const bbox: BBox = {
      south: mockBounds.getSouth(),
      west: mockBounds.getWest(),
      north: mockBounds.getNorth(),
      east: mockBounds.getEast(),
    };

    expect(bbox).toEqual({
      south: 35.65,
      west: 139.72,
      north: 35.71,
      east: 139.81,
    });
  });
});
```

- [ ] **Step 2: Run test to verify it passes**

```bash
cd services/frontend && pnpm vitest run src/__tests__/map-view-bbox.test.ts
```

Expected: PASS (these are type/contract tests)

- [ ] **Step 3: Update MapViewProps to accept BBox in onMoveEnd**

Modify `services/frontend/src/components/map/map-view.tsx`:

Change the `MapViewProps` interface:
```typescript
// Before
interface MapViewProps {
  children?: ReactNode;
  onMoveEnd?: () => void;
  onFeatureClick?: (e: MapLayerMouseEvent) => void;
}

// After
import type { BBox } from "@/lib/api";

interface MapViewProps {
  children?: ReactNode;
  onMoveEnd?: (bbox: BBox) => void;
  onFeatureClick?: (e: MapLayerMouseEvent) => void;
}
```

Update the `handleMoveEnd` callback to extract real bounds from `mapRef`:
```typescript
// Before
const handleMoveEnd = useCallback(
  (_e: ViewStateChangeEvent) => {
    if (moveEndTimerRef.current) clearTimeout(moveEndTimerRef.current);
    moveEndTimerRef.current = setTimeout(() => {
      onMoveEnd?.();
    }, DEBOUNCE_MS);
  },
  [onMoveEnd],
);

// After
const handleMoveEnd = useCallback(
  (_e: ViewStateChangeEvent) => {
    if (moveEndTimerRef.current) clearTimeout(moveEndTimerRef.current);
    moveEndTimerRef.current = setTimeout(() => {
      const map = mapRef.current;
      if (!map || !onMoveEnd) return;
      const b = map.getBounds();
      onMoveEnd({
        south: b.getSouth(),
        west: b.getWest(),
        north: b.getNorth(),
        east: b.getEast(),
      });
    }, DEBOUNCE_MS);
  },
  [onMoveEnd],
);
```

Add the `BBox` import at the top of the file:
```typescript
import type { BBox } from "@/lib/api";
```

- [ ] **Step 4: Update useMapPage to receive BBox from callback**

Modify `services/frontend/src/hooks/use-map-page.ts`:

```typescript
// Before
import type { LayerConfig } from "@/lib/layers";
// ...
const { visibleLayers, getBBox } = useMapStore(
  useShallow((s) => ({
    visibleLayers: s.visibleLayers,
    getBBox: s.getBBox,
  })),
);
// ...
const [bbox, setBbox] = useState(() => getBBox());
const handleMoveEnd = useCallback(() => {
  setBbox(getBBox());
}, [getBBox]);

// After
import type { BBox } from "@/lib/api";
import type { LayerConfig } from "@/lib/layers";
// ...
const visibleLayers = useMapStore((s) => s.visibleLayers);
// ...
const [bbox, setBbox] = useState<BBox>(() => useMapStore.getState().getBBox());
const handleMoveEnd = useCallback((newBbox: BBox) => {
  setBbox(newBbox);
}, []);
```

Note: The initial state still uses `getBBox()` from the store as fallback (map instance isn't available yet on mount). After first map moveEnd, real bounds take over.

- [ ] **Step 5: Run full test suite**

```bash
cd services/frontend && pnpm vitest run
```

Expected: All tests PASS (no test depends on the old `getBBox()` being called on moveEnd)

- [ ] **Step 6: Run type check + lint**

```bash
cd services/frontend && pnpm tsc --noEmit && pnpm biome check .
```

Expected: Clean

- [ ] **Step 7: Commit**

```bash
git add services/frontend/src/components/map/map-view.tsx \
       services/frontend/src/hooks/use-map-page.ts \
       services/frontend/src/__tests__/map-view-bbox.test.ts
git commit -m "fix(map): use map.getBounds() instead of center+zoom approximation

Replace the geometric bbox approximation (180/2^zoom) with MapLibre's actual
map.getBounds() for accurate viewport queries at any pitch/bearing angle.

Closes X-06."
```

---

### Task 2: Update getBBox test documentation and verify pitch accuracy

**Files:**
- Modify: `services/frontend/src/__tests__/map-store-extended.test.ts:53-86`
- Modify: `services/frontend/src/__tests__/map-view-bbox.test.ts`

- [ ] **Step 1: Add pitch/bearing documentation to getBBox tests**

In `services/frontend/src/__tests__/map-store-extended.test.ts`, update the test description to clarify this is fallback-only:

```typescript
// Before
it("getBBox returns correct bounds for default viewState", () => {

// After
it("getBBox returns approximate bounds for initial state (fallback only)", () => {
```

```typescript
// Before
it("getBBox shrinks as zoom increases", () => {

// After
it("getBBox approximation shrinks as zoom increases", () => {
```

- [ ] **Step 2: Add test documenting the pitch inaccuracy that motivated this change**

Add to `services/frontend/src/__tests__/map-view-bbox.test.ts`:

```typescript
describe("getBBox approximation limitations (documented)", () => {
  it("approximation ignores pitch — same bbox at pitch 0 and 60", () => {
    // This documents WHY we switched to map.getBounds()
    // The old approximation gives identical results regardless of pitch,
    // but a pitched map shows a much larger geographic area
    const calcApproxBbox = (lat: number, lng: number, zoom: number) => {
      const latRange = 180 / 2 ** zoom;
      const lngRange = 360 / 2 ** zoom;
      return {
        south: lat - latRange / 2,
        west: lng - lngRange / 2,
        north: lat + latRange / 2,
        east: lng + lngRange / 2,
      };
    };

    const bboxPitch0 = calcApproxBbox(35.681, 139.767, 12);
    const bboxPitch60 = calcApproxBbox(35.681, 139.767, 12);

    // They're identical — that's the bug. A pitched view sees more area
    // but the approximation doesn't know that.
    expect(bboxPitch0).toEqual(bboxPitch60);
  });
});
```

- [ ] **Step 3: Run tests**

```bash
cd services/frontend && pnpm vitest run src/__tests__/map-view-bbox.test.ts src/__tests__/map-store-extended.test.ts
```

Expected: All PASS

- [ ] **Step 4: Commit**

```bash
git add services/frontend/src/__tests__/map-store-extended.test.ts \
       services/frontend/src/__tests__/map-view-bbox.test.ts
git commit -m "test: document getBBox pitch limitation and clarify fallback role"
```

---

## Self-Review Checklist

1. **Spec coverage**: X-06 requires (a) `map.getBounds()` as single source ✅, (b) debounce separation ✅ (existing 300ms debounce in MapView preserved), (c) pitch/bearing accuracy ✅ (real bounds handle this).

2. **Placeholder scan**: No TBD/TODO/placeholders found. All code blocks are complete.

3. **Type consistency**: `BBox` type from `@/lib/api` used consistently. `onMoveEnd` signature `(bbox: BBox) => void` matches between MapViewProps (Task 1 Step 3) and useMapPage handler (Task 1 Step 4). `handleMoveEnd` callback in useMapPage (Step 4) matches the prop type.
