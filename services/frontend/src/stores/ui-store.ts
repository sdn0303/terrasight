import { create } from "zustand";
import { devtools } from "zustand/middleware";

type Locale = "ja" | "en";

import type { ThemeId } from "@/lib/themes";

/** Overlay state types (Phase 1+). */
export type LeftPanelKind = "finder" | "layers" | "themes";
export type BottomSheetKind = "opportunities";
export type DrawerTab = "intel" | "trend" | "risk" | "infra" | "compare";
export type BaseMap = "light" | "dark" | "satellite";

export type InsightContext =
  | null
  | { kind: "point"; lat: number; lng: number }
  | { kind: "property"; id: string; lat: number; lng: number };

export type ComparePoint = {
  lat: number;
  lng: number;
  address: string;
};

interface UIState {
  // ─── Overlay state ─────────
  leftPanel: LeftPanelKind | null;
  setLeftPanel: (p: LeftPanelKind | null) => void;
  toggleLeftPanel: (p: LeftPanelKind) => void;

  bottomSheet: BottomSheetKind | null;
  setBottomSheet: (b: BottomSheetKind | null) => void;
  bottomSheetHeightPct: number;
  setBottomSheetHeightPct: (h: number) => void;

  insight: InsightContext;
  setInsight: (c: InsightContext) => void;
  activeTab: DrawerTab;
  setActiveTab: (t: DrawerTab) => void;

  settingsOpen: boolean;
  setSettingsOpen: (o: boolean) => void;

  baseMap: BaseMap;
  setBaseMap: (m: BaseMap) => void;

  // ─── Compare points ───────
  comparePoints: ComparePoint[];
  addComparePoint: (point: ComparePoint) => void;
  removeComparePoint: (index: number) => void;
  resetCompare: () => void;

  locale: Locale;
  setLocale: (locale: Locale) => void;

  activeThemes: Set<ThemeId>;
  toggleTheme: (themeId: ThemeId) => void;
  clearThemes: () => void;
}

export const useUIStore = create<UIState>()(
  devtools(
    (set) => ({
      // Overlay state
      leftPanel: null,
      setLeftPanel: (p) => set({ leftPanel: p }),
      toggleLeftPanel: (p) =>
        set((s) => ({ leftPanel: s.leftPanel === p ? null : p })),

      bottomSheet: null,
      setBottomSheet: (b) => set({ bottomSheet: b }),
      bottomSheetHeightPct: 40,
      setBottomSheetHeightPct: (h) =>
        set({ bottomSheetHeightPct: Math.max(20, Math.min(80, h)) }),

      insight: null,
      setInsight: (c) => set({ insight: c }),
      activeTab: "intel",
      setActiveTab: (t) => set({ activeTab: t }),

      settingsOpen: false,
      setSettingsOpen: (o) => set({ settingsOpen: o }),

      baseMap: "light",
      setBaseMap: (m) => set({ baseMap: m }),

      // Compare points
      comparePoints: [],
      addComparePoint: (point) =>
        set((state) => {
          if (state.comparePoints.length >= 3) return state;
          return { comparePoints: [...state.comparePoints, point] };
        }),
      removeComparePoint: (index) =>
        set((state) => ({
          comparePoints: state.comparePoints.filter((_, i) => i !== index),
        })),
      resetCompare: () => set({ comparePoints: [] }),

      locale: "ja",
      setLocale: (locale) => set({ locale }),

      activeThemes: new Set<ThemeId>(),
      toggleTheme: (themeId) =>
        set((state) => {
          const next = new Set(state.activeThemes);
          if (next.has(themeId)) {
            next.delete(themeId);
          } else {
            next.add(themeId);
          }
          return { activeThemes: next };
        }),
      clearThemes: () => set({ activeThemes: new Set<ThemeId>() }),
    }),
    { name: "ui-store" },
  ),
);
