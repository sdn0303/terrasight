import { create } from "zustand";
import { devtools } from "zustand/middleware";
import type { Locale } from "@/i18n/config";
import type { ThemeId } from "@/lib/themes";

export type AppMode = "explore" | "compare";

export type ComparePoint = {
  lat: number;
  lng: number;
  address: string;
};

interface UIState {
  mode: AppMode;
  setMode: (mode: AppMode) => void;
  locale: Locale;
  setLocale: (locale: Locale) => void;
  activeThemes: Set<ThemeId>;
  toggleTheme: (themeId: ThemeId) => void;
  clearThemes: () => void;
  layerSettingsOpen: boolean;
  toggleLayerSettings: () => void;
  comparePoints: ComparePoint[];
  addComparePoint: (point: ComparePoint) => void;
  removeComparePoint: (index: number) => void;
  resetCompare: () => void;
}

export const useUIStore = create<UIState>()(
  devtools(
    (set) => ({
      mode: "explore",
      setMode: (mode) => set({ mode }),

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

      layerSettingsOpen: true,
      toggleLayerSettings: () =>
        set((state) => ({ layerSettingsOpen: !state.layerSettingsOpen })),

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
    }),
    { name: "ui-store" },
  ),
);
