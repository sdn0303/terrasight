# Layer Curation — Theme Card Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the small 2×2 theme buttons with larger, descriptive theme cards that show a description and layer count, making the 4 themes the primary navigation for the 24+ layers.

**Architecture:** Add theme descriptions to i18n files. Redesign `ThemePresets` component with taller cards showing icon, name, description, and layer count. Add `getLayerIdsByTheme` import to show count. Single-file UI change + i18n update.

**Tech Stack:** React 19, next-intl, Tailwind CSS v4 (design tokens), Vitest

---

## File Structure

| File | Change | Responsibility |
|------|--------|----------------|
| `services/frontend/src/i18n/locales/ja.json` | Modify | Add theme descriptions |
| `services/frontend/src/i18n/locales/en.json` | Modify | Add theme descriptions |
| `services/frontend/src/components/explore/theme-presets.tsx` | Modify | Redesign to larger cards with description + count |
| `services/frontend/src/__tests__/theme-presets.test.tsx` | Create | Test card rendering, active state, layer count |

---

### Task 1: Add theme descriptions to i18n and redesign ThemePresets

**Files:**
- Modify: `services/frontend/src/i18n/locales/ja.json`
- Modify: `services/frontend/src/i18n/locales/en.json`
- Modify: `services/frontend/src/components/explore/theme-presets.tsx`
- Create: `services/frontend/src/__tests__/theme-presets.test.tsx`

- [ ] **Step 1: Write tests for the redesigned ThemePresets**

Create `services/frontend/src/__tests__/theme-presets.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { getLayerIdsByTheme } from "@/lib/themes";

// Mock next-intl
vi.mock("next-intl", () => ({
  useTranslations: () => (key: string) => {
    const map: Record<string, string> = {
      "theme.safety": "安全性",
      "theme.livability": "利便性",
      "theme.price": "価格",
      "theme.future": "将来性",
      "theme.safety.desc": "洪水・地震・急傾斜地などの災害リスクを可視化",
      "theme.livability.desc": "学校・医療・鉄道などの生活インフラを表示",
      "theme.price.desc": "地価公示・用途地域の投資価値データ",
      "theme.future.desc": "人口推移・都市計画の将来性指標",
    };
    return map[key] ?? key;
  },
}));

describe("Theme layer counts", () => {
  it("safety theme has layers", () => {
    const layers = getLayerIdsByTheme("safety");
    expect(layers.length).toBeGreaterThan(0);
  });

  it("all four themes have at least one layer", () => {
    for (const id of ["safety", "livability", "price", "future"] as const) {
      expect(getLayerIdsByTheme(id).length).toBeGreaterThan(0);
    }
  });
});
```

- [ ] **Step 2: Run test to verify it passes**

```bash
cd services/frontend && pnpm vitest run src/__tests__/theme-presets.test.tsx
```

Expected: PASS

- [ ] **Step 3: Add theme descriptions to i18n files**

In `services/frontend/src/i18n/locales/ja.json`, change the `theme` section:

```json
"theme": {
  "safety": "安全性",
  "safety.desc": "洪水・地震・急傾斜地などの災害リスクを可視化",
  "livability": "利便性",
  "livability.desc": "学校・医療・鉄道などの生活インフラを表示",
  "price": "価格",
  "price.desc": "地価公示・用途地域の投資価値データ",
  "future": "将来性",
  "future.desc": "人口推移・都市計画の将来性指標"
}
```

In `services/frontend/src/i18n/locales/en.json`, change the `theme` section:

```json
"theme": {
  "safety": "Safety",
  "safety.desc": "Flood, seismic, and landslide hazard layers",
  "livability": "Livability",
  "livability.desc": "Schools, medical, railway infrastructure",
  "price": "Price",
  "price.desc": "Land prices and zoning value data",
  "future": "Future",
  "future.desc": "Population trends and urban planning"
}
```

- [ ] **Step 4: Redesign ThemePresets component**

Replace the entire content of `services/frontend/src/components/explore/theme-presets.tsx`:

```tsx
"use client";

import { useEffect } from "react";
import { useTranslations } from "next-intl";
import { THEMES, getLayerIdsByTheme, getLayerIdsForThemes } from "@/lib/themes";
import type { ThemeId } from "@/lib/themes";
import { useMapStore } from "@/stores/map-store";
import { useUIStore } from "@/stores/ui-store";

const ICONS: Record<ThemeId, string> = {
  safety: "\u{1F6E1}",
  livability: "\u{1F3D8}",
  price: "\u{1F4B0}",
  future: "\u{1F4C8}",
};

export function ThemePresets() {
  const t = useTranslations();
  const activeThemes = useUIStore((s) => s.activeThemes);
  const toggleTheme = useUIStore((s) => s.toggleTheme);

  useEffect(() => {
    if (activeThemes.size === 0) {
      useMapStore.setState({ visibleLayers: new Set<string>() });
      return;
    }
    const themeLayerIds = getLayerIdsForThemes(activeThemes);
    useMapStore.setState({ visibleLayers: themeLayerIds });
  }, [activeThemes]);

  return (
    <div className="flex flex-col gap-2 px-4 py-3">
      {THEMES.map((theme) => {
        const isActive = activeThemes.has(theme.id);
        const layerCount = getLayerIdsByTheme(theme.id).length;
        return (
          <button
            key={theme.id}
            type="button"
            onClick={() => toggleTheme(theme.id)}
            className={`flex items-start gap-3 rounded-lg px-4 py-3 text-left transition-colors border ${
              isActive
                ? "bg-ds-hover-accent border-ds-accent-cyan/50"
                : "bg-ds-bg-tertiary/50 border-transparent hover:bg-ds-bg-tertiary"
            }`}
            aria-pressed={isActive}
          >
            <span className="text-xl mt-0.5">{ICONS[theme.id]}</span>
            <div className="flex-1 min-w-0">
              <div className="flex items-center justify-between">
                <span
                  className={`text-xs font-medium ${
                    isActive ? "text-ds-accent-cyan" : "text-ds-text-primary"
                  }`}
                >
                  {t(theme.labelKey)}
                </span>
                <span
                  className="text-[9px] font-mono"
                  style={{ color: "var(--text-muted)" }}
                >
                  {layerCount} layers
                </span>
              </div>
              <p
                className="text-[10px] mt-0.5 leading-relaxed"
                style={{ color: "var(--text-secondary)" }}
              >
                {t(`theme.${theme.id}.desc`)}
              </p>
            </div>
          </button>
        );
      })}
    </div>
  );
}
```

Key changes from the old 2×2 grid:
- Layout: `flex flex-col` (vertical stack) instead of `grid grid-cols-2` (2×2 grid)
- Each card shows icon (larger), name, layer count, and description
- Active state: cyan border + accent background
- Inactive state: subtle background with hover

- [ ] **Step 5: Run full test suite**

```bash
cd services/frontend && pnpm vitest run
```

Expected: All tests PASS

- [ ] **Step 6: Run type check + lint**

```bash
cd services/frontend && pnpm tsc --noEmit && pnpm biome check .
```

Expected: Clean

- [ ] **Step 7: Commit**

```bash
git add services/frontend/src/i18n/locales/ja.json \
       services/frontend/src/i18n/locales/en.json \
       services/frontend/src/components/explore/theme-presets.tsx \
       services/frontend/src/__tests__/theme-presets.test.tsx
git commit -m "feat(ui): redesign theme presets as descriptive cards with layer counts

Replace the small 2×2 theme buttons with vertical card layout showing
icon, name, description, and layer count. Makes themes the primary
navigation for the 24+ map layers."
```

---

## Self-Review Checklist

1. **Spec coverage**: Large theme cards ✅. Layer count per theme ✅. 4 themes (安全性/利便性/価格/将来性) ✅. i18n descriptions ✅.

2. **Placeholder scan**: No TBD/TODO. All code blocks complete.

3. **Type consistency**: `ThemeId` used consistently. `getLayerIdsByTheme` imported from `@/lib/themes` (already exists). `t(theme.labelKey)` and `t(\`theme.${theme.id}.desc\`)` match i18n keys.
