import { create } from "zustand";
import { devtools } from "zustand/middleware";

type ComparePoint = {
  lat: number;
  lng: number;
  address: string;
} | null;

interface UIState {
  compareMode: boolean;
  comparePointA: ComparePoint;
  comparePointB: ComparePoint;
  layerPanelOpen: boolean;
  enterCompareMode: () => void;
  exitCompareMode: () => void;
  setComparePoint: (point: { lat: number; lng: number; address: string }) => void;
  toggleLayerPanel: () => void;
}

export const useUIStore = create<UIState>()(
  devtools(
    (set, get) => ({
      compareMode: false,
      comparePointA: null,
      comparePointB: null,
      layerPanelOpen: true,

      enterCompareMode: () =>
        set({ compareMode: true, comparePointA: null, comparePointB: null }),

      exitCompareMode: () =>
        set({ compareMode: false, comparePointA: null, comparePointB: null }),

      setComparePoint: (point) => {
        const { comparePointA } = get();
        if (comparePointA === null) {
          set({ comparePointA: point });
        } else {
          set({ comparePointB: point });
        }
      },

      toggleLayerPanel: () =>
        set((state) => ({ layerPanelOpen: !state.layerPanelOpen })),
    }),
    { name: "ui-store" },
  ),
);
