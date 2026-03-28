# Error State Visibility Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make three silent error states visible to users: WASM init failure, area data API errors, and zoom-too-low overlay on the map.

**Architecture:** Add error indicators to StatusBar (WASM + area data errors) and a zoom overlay on the map container. No new toast library — follow existing patterns (inline status indicators, ARIA roles). Expose `isError` from `useAreaData` and `wasmError` from `useMapPage` through to the UI.

**Tech Stack:** React 19, TanStack Query v5, Zustand, Vitest

---

## File Structure

| File | Change | Responsibility |
|------|--------|----------------|
| `services/frontend/src/hooks/use-map-page.ts` | Modify | Expose `areaDataError` from useAreaData |
| `services/frontend/src/components/status-bar.tsx` | Modify | Display WASM error + area data error indicators |
| `services/frontend/src/app/page.tsx` | Modify | Pass error props to StatusBar, add zoom overlay |
| `services/frontend/src/__tests__/status-bar.test.tsx` | Create | Test error indicator rendering |

---

### Task 1: Expose area data error and add error indicators to StatusBar + zoom overlay

**Files:**
- Modify: `services/frontend/src/hooks/use-map-page.ts`
- Modify: `services/frontend/src/components/status-bar.tsx`
- Modify: `services/frontend/src/app/page.tsx`
- Create: `services/frontend/src/__tests__/status-bar.test.tsx`

- [ ] **Step 1: Write tests for StatusBar error indicators**

Create `services/frontend/src/__tests__/status-bar.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { StatusBar } from "@/components/status-bar";

const defaultProps = {
  lat: 35.681,
  lng: 139.767,
  zoom: 12,
  isLoading: false,
  isDemoMode: false,
};

describe("StatusBar error indicators", () => {
  it("shows WASM error indicator when wasmError is true", () => {
    render(<StatusBar {...defaultProps} wasmError={true} />);
    expect(screen.getByText(/WASM/)).toBeInTheDocument();
  });

  it("hides WASM error indicator when wasmError is false", () => {
    render(<StatusBar {...defaultProps} wasmError={false} />);
    expect(screen.queryByText(/WASM/)).not.toBeInTheDocument();
  });

  it("shows area data error indicator when areaDataError is true", () => {
    render(<StatusBar {...defaultProps} areaDataError={true} />);
    expect(screen.getByText(/データ取得エラー/)).toBeInTheDocument();
  });

  it("hides area data error indicator when areaDataError is false", () => {
    render(<StatusBar {...defaultProps} areaDataError={false} />);
    expect(screen.queryByText(/データ取得エラー/)).not.toBeInTheDocument();
  });

  it("shows zoom warning when isZoomTooLow is true", () => {
    render(<StatusBar {...defaultProps} isZoomTooLow={true} />);
    expect(screen.getByText(/ズームイン/)).toBeInTheDocument();
  });

  it("shows both WASM and area data errors simultaneously", () => {
    render(
      <StatusBar {...defaultProps} wasmError={true} areaDataError={true} />,
    );
    expect(screen.getByText(/WASM/)).toBeInTheDocument();
    expect(screen.getByText(/データ取得エラー/)).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd services/frontend && pnpm vitest run src/__tests__/status-bar.test.tsx
```

Expected: FAIL — StatusBar doesn't accept `wasmError`, `areaDataError`, or `isZoomTooLow` props yet.

- [ ] **Step 3: Update StatusBar to accept and display error props**

Modify `services/frontend/src/components/status-bar.tsx`:

Add new props to the interface:
```typescript
interface StatusBarProps {
  lat: number;
  lng: number;
  zoom: number;
  isLoading: boolean;
  isDemoMode: boolean;
  truncatedLayers?: TruncationInfo[];
  wasmError?: boolean;
  areaDataError?: boolean;
  isZoomTooLow?: boolean;
}
```

Add the props to the destructuring and render error indicators after the existing loading/demo indicators:
```tsx
export function StatusBar({
  lat,
  lng,
  zoom,
  isLoading,
  isDemoMode,
  truncatedLayers,
  wasmError,
  areaDataError,
  isZoomTooLow,
}: StatusBarProps) {
  return (
    <div ...>
      {/* ... existing lat/lng/zoom/demo/loading ... */}
      {wasmError && (
        <span role="alert" style={{ color: "var(--accent-danger)" }}>
          ✕ WASM エンジン初期化失敗
        </span>
      )}
      {areaDataError && (
        <span role="alert" style={{ color: "var(--accent-danger)" }}>
          ✕ データ取得エラー
        </span>
      )}
      {isZoomTooLow && (
        <span style={{ color: "var(--accent-warning)" }}>
          ⚠ ズームインでデータ表示
        </span>
      )}
      {/* ... existing truncation warnings ... */}
    </div>
  );
}
```

- [ ] **Step 4: Expose areaDataError from useMapPage**

Modify `services/frontend/src/hooks/use-map-page.ts`:

Change the `useAreaData` destructuring to include `isError`:
```typescript
// Before
const { data: areaData, isLoading } = useAreaData(bbox, layers, viewState.zoom);

// After
const { data: areaData, isLoading, isError: areaDataError } = useAreaData(bbox, layers, viewState.zoom);
```

Add `areaDataError` to the return object:
```typescript
return {
  // ... existing fields ...
  areaDataError,
};
```

- [ ] **Step 5: Wire error props in page.tsx**

Modify `services/frontend/src/app/page.tsx`:

Pass error props to StatusBar:
```tsx
<StatusBar
  lat={page.viewState.latitude}
  lng={page.viewState.longitude}
  zoom={page.viewState.zoom}
  isLoading={page.isLoading}
  isDemoMode={page.isDemoMode}
  truncatedLayers={page.truncatedLayers}
  wasmError={page.wasmError}
  areaDataError={page.areaDataError}
  isZoomTooLow={page.isZoomTooLow}
/>
```

Add a zoom overlay on the map area (inside the map container div, after MapView):
```tsx
{page.isZoomTooLow && (
  <div
    className="absolute inset-0 flex items-center justify-center pointer-events-none"
    style={{ zIndex: 10 }}
  >
    <div
      className="rounded-lg px-6 py-3 text-center"
      style={{
        background: "rgba(12, 12, 20, 0.75)",
        border: "1px solid var(--border-primary)",
        fontFamily: "var(--font-mono)",
        fontSize: "12px",
        color: "var(--text-secondary)",
      }}
    >
      ズームインしてデータを表示
    </div>
  </div>
)}
```

- [ ] **Step 6: Run tests**

```bash
cd services/frontend && pnpm vitest run
```

Expected: All tests PASS including new StatusBar tests.

- [ ] **Step 7: Run type check + lint**

```bash
cd services/frontend && pnpm tsc --noEmit && pnpm biome check .
```

Expected: Clean

- [ ] **Step 8: Commit**

```bash
git add services/frontend/src/hooks/use-map-page.ts \
       services/frontend/src/components/status-bar.tsx \
       services/frontend/src/app/page.tsx \
       services/frontend/src/__tests__/status-bar.test.tsx
git commit -m "feat(ui): show WASM, area data, and zoom error states to user

Previously all three error states were silent:
- WASM spatial engine init failure: logged but never shown
- Area data API errors: isError not extracted from React Query
- Zoom too low: only visible in land price slider

Now visible via StatusBar indicators + zoom overlay on map."
```

---

## Self-Review Checklist

1. **Spec coverage**: (a) WASM toast/indicator ✅ via StatusBar, (b) area data error ✅ via StatusBar, (c) zoom overlay ✅ on map container. All three from TODOS.md addressed.

2. **Placeholder scan**: No TBD/TODO. All code blocks complete.

3. **Type consistency**: `wasmError: boolean` in useMapPage return, `wasmError?: boolean` in StatusBarProps. `areaDataError` from React Query's `isError` (boolean). `isZoomTooLow` already exists in useMapPage return.
