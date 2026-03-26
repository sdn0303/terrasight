import { create } from "zustand";
import { devtools } from "zustand/middleware";
import type { Locale } from "@/i18n/config";
import type { ThemeId } from "@/lib/themes";

export type AppMode = "explore" | "compare";

type ComparePoint = {
  lat: number;
  lng: number;
  address: string;
} | null;

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
  comparePointA: ComparePoint;
  comparePointB: ComparePoint;
  setComparePoint: (point: {
    lat: number;
    lng: number;
    address: string;
  }) => void;
  resetCompare: () => void;
}

export const useUIStore = create<UIState>()(
  devtools(
    (set, get) => ({
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

      comparePointA: null,
      comparePointB: null,

      setComparePoint: (point) => {
        const { comparePointA } = get();
        if (comparePointA === null) {
          set({ comparePointA: point });
        } else {
          set({ comparePointB: point });
        }
      },

      resetCompare: () => set({ comparePointA: null, comparePointB: null }),
    }),
    { name: "ui-store" },
  ),
);
