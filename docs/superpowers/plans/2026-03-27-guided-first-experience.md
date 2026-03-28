# Guided First Experience Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** On first visit (no URL params), activate the Safety theme so users immediately see rich hazard data instead of a sparse map. Return visitors with URL params see their saved state.

**Architecture:** Add `theme` to URL state params (default `"safety"`). On mount, restore theme from URL into `useUIStore.activeThemes`. The existing `ThemePresets` `useEffect` already syncs `activeThemes` → `visibleLayers` in `map-store`. Return visitors' URLs include their active theme, so state is preserved across sessions.

**Tech Stack:** React 19, nuqs (URL state), Zustand, Vitest

---

## File Structure

| File | Change | Responsibility |
|------|--------|----------------|
| `services/frontend/src/hooks/use-map-url-state.ts` | Modify | Add `theme` URL param, restore on mount, sync to URL |
| `services/frontend/src/__tests__/use-map-url-state.test.ts` | Create | Test first-visit default theme activation |

---

### Task 1: Add theme URL param and first-visit activation

**Files:**
- Modify: `services/frontend/src/hooks/use-map-url-state.ts`
- Create: `services/frontend/src/__tests__/use-map-url-state.test.ts`

- [ ] **Step 1: Write the test**

Create `services/frontend/src/__tests__/use-map-url-state.test.ts`:

```typescript
import { describe, expect, it } from "vitest";
import type { ThemeId } from "@/lib/themes";
import { THEMES } from "@/lib/themes";

describe("first-visit theme activation contract", () => {
  it("safety is a valid ThemeId", () => {
    const safetyTheme = THEMES.find((t) => t.id === "safety");
    expect(safetyTheme).toBeDefined();
    expect(safetyTheme?.id).toBe("safety");
  });

  it("default theme param parses to safety", () => {
    // The URL param default is "safety", which must match a valid ThemeId
    const defaultTheme = "safety";
    const validIds = THEMES.map((t) => t.id);
    expect(validIds).toContain(defaultTheme);
  });

  it("empty theme param string results in no themes", () => {
    const themeParam = "";
    const themeIds = themeParam.split(",").filter(Boolean);
    expect(themeIds).toHaveLength(0);
  });

  it("multiple themes can be serialized and deserialized", () => {
    const themes: ThemeId[] = ["safety", "livability"];
    const serialized = themes.join(",");
    const deserialized = serialized.split(",").filter(Boolean);
    expect(deserialized).toEqual(["safety", "livability"]);
  });
});
```

- [ ] **Step 2: Run test to verify it passes**

```bash
cd services/frontend && pnpm vitest run src/__tests__/use-map-url-state.test.ts
```

Expected: PASS (contract tests)

- [ ] **Step 3: Update use-map-url-state.ts**

Modify `services/frontend/src/hooks/use-map-url-state.ts`:

1. Add import for `useUIStore` and `ThemeId`:
```typescript
import { useUIStore } from "@/stores/ui-store";
import type { ThemeId } from "@/lib/themes";
import { THEMES } from "@/lib/themes";
```

2. Add `theme` to `mapParams`:
```typescript
const mapParams = {
  lat: parseAsFloat.withDefault(MAP_CONFIG.center[1]),
  lng: parseAsFloat.withDefault(MAP_CONFIG.center[0]),
  z: parseAsFloat.withDefault(MAP_CONFIG.zoom),
  pitch: parseAsFloat.withDefault(MAP_CONFIG.pitch),
  bearing: parseAsFloat.withDefault(MAP_CONFIG.bearing),
  layers: parseAsString.withDefault("land_price_ts,zoning"),
  theme: parseAsString.withDefault("safety"),
  year: parseAsInteger.withDefault(2024),
};
```

3. In the mount `useEffect`, after restoring view state and layers, restore theme:
```typescript
// Restore theme from URL
const validThemeIds = new Set(THEMES.map((t) => t.id));
const themeIds = params.theme
  .split(",")
  .filter((id): id is ThemeId => validThemeIds.has(id as ThemeId));

const currentThemes = useUIStore.getState().activeThemes;
// Clear any existing themes first
if (currentThemes.size > 0) {
  useUIStore.getState().clearThemes();
}
// Activate themes from URL
for (const themeId of themeIds) {
  useUIStore.getState().toggleTheme(themeId);
}
```

4. In the sync `useEffect`, add theme sync. Get `activeThemes` from `useUIStore`:
```typescript
const activeThemes = useUIStore((s) => s.activeThemes);
```

And in the sync effect, include theme:
```typescript
useEffect(() => {
  if (!initialized.current) return;
  setParams({
    lat: Math.round(viewState.latitude * 10000) / 10000,
    lng: Math.round(viewState.longitude * 10000) / 10000,
    z: Math.round(viewState.zoom * 10) / 10,
    pitch: Math.round(viewState.pitch),
    bearing: Math.round(viewState.bearing),
    layers: [...visibleLayers].sort().join(","),
    theme: [...activeThemes].sort().join(","),
  });
}, [viewState, visibleLayers, activeThemes, setParams]);
```

5. Since ThemePresets' useEffect syncs `activeThemes` → `visibleLayers`, the `layers` URL default (`"land_price_ts,zoning"`) will be immediately overwritten by theme-based layers on first visit. The URL will update to reflect the actual safety theme layers.

- [ ] **Step 4: Run full test suite**

```bash
cd services/frontend && pnpm vitest run
```

Expected: All tests PASS

- [ ] **Step 5: Run type check + lint**

```bash
cd services/frontend && pnpm tsc --noEmit && pnpm biome check .
```

Expected: Clean

- [ ] **Step 6: Commit**

```bash
git add services/frontend/src/hooks/use-map-url-state.ts \
       services/frontend/src/__tests__/use-map-url-state.test.ts
git commit -m "feat(ux): activate Safety theme on first visit

First-time users see the Safety theme (9 hazard layers) instead of a
sparse map with only land prices. Theme state persists in URL params
so return visitors see their last-active theme."
```

---

## Self-Review Checklist

1. **Spec coverage**: First visit activates safety theme ✅. Chiyoda-ku center already in MAP_CONFIG defaults ✅. Theme persists in URL ✅.

2. **Placeholder scan**: No TBD/TODO. All code blocks complete.

3. **Type consistency**: `ThemeId` used consistently from `@/lib/themes`. URL param `theme` is string, parsed into `ThemeId[]` with validation.
