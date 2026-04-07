import { create } from "zustand";
import { devtools } from "zustand/middleware";
import { LAYERS } from "@/lib/layers";

interface ViewState {
  latitude: number;
  longitude: number;
  zoom: number;
  pitch: number;
  bearing: number;
}

interface SelectedFeature {
  layerId: string;
  properties: Record<string, unknown>;
  coordinates: [number, number]; // [lng, lat] per RFC 7946
}

export interface SelectedArea {
  code: string; // Administrative code (e.g., "13" for Tokyo, "13105" for Bunkyo)
  name: string; // Display name
  level: "prefecture" | "municipality";
  bbox: { south: number; west: number; north: number; east: number };
}

export type WeightPreset =
  | "balance"
  | "investment"
  | "residential"
  | "disaster";

export interface AnalysisPoint {
  lat: number;
  lng: number;
  address?: string;
}

interface MapState {
  viewState: ViewState;
  visibleLayers: Set<string>;
  selectedFeature: SelectedFeature | null;
  selectedArea: SelectedArea | null;
  analysisPoint: AnalysisPoint | null;
  weightPreset: WeightPreset;
  analysisRadius: number;
  setViewState: (viewState: ViewState) => void;
  toggleLayer: (layerId: string) => void;
  applyThemeLayers: (themeLayers: Set<string>) => void;
  resetToDefaults: () => void;
  selectFeature: (feature: SelectedFeature | null) => void;
  selectArea: (area: SelectedArea | null) => void;
  getBBox: () => { south: number; west: number; north: number; east: number };
  setAnalysisPoint: (point: AnalysisPoint | null) => void;
  setWeightPreset: (preset: WeightPreset) => void;
  setAnalysisRadius: (radius: number) => void;
}

const defaultVisibleLayers = new Set(
  LAYERS.filter((l) => l.defaultEnabled).map((l) => l.id),
);

export const useMapStore = create<MapState>()(
  devtools(
    (set, get) => ({
      viewState: {
        latitude: 35.681,
        longitude: 139.767,
        zoom: 12,
        pitch: 45,
        bearing: 0,
      },
      visibleLayers: defaultVisibleLayers,
      selectedFeature: null,
      selectedArea: null,
      analysisPoint: null,
      weightPreset: "balance",
      analysisRadius: 500,

      setViewState: (viewState) => set({ viewState }),

      toggleLayer: (layerId) =>
        set((state) => {
          const next = new Set(state.visibleLayers);
          if (next.has(layerId)) {
            next.delete(layerId);
          } else {
            next.add(layerId);
          }
          return { visibleLayers: next };
        }),

      applyThemeLayers: (themeLayers) =>
        set((state) => {
          // When theme is empty, restore defaults only (no manual preservation)
          if (themeLayers.size === 0) {
            return { visibleLayers: new Set(defaultVisibleLayers) };
          }
          // Union: defaults + currently visible (manually toggled) layers + theme layers
          const next = new Set(defaultVisibleLayers);
          for (const id of state.visibleLayers) {
            next.add(id);
          }
          for (const id of themeLayers) {
            next.add(id);
          }
          return { visibleLayers: next };
        }),

      resetToDefaults: () =>
        set({ visibleLayers: new Set(defaultVisibleLayers) }),

      selectFeature: (feature) => set({ selectedFeature: feature }),

      selectArea: (area) => set({ selectedArea: area }),

      setAnalysisPoint: (point) => set({ analysisPoint: point }),

      setWeightPreset: (preset) => set({ weightPreset: preset }),

      setAnalysisRadius: (radius) => set({ analysisRadius: radius }),

      getBBox: () => {
        const { latitude, longitude, zoom } = get().viewState;
        const latRange = 180 / 2 ** zoom;
        const lngRange = 360 / 2 ** zoom;
        return {
          south: latitude - latRange / 2,
          west: longitude - lngRange / 2,
          north: latitude + latRange / 2,
          east: longitude + lngRange / 2,
        };
      },
    }),
    { name: "map-store" },
  ),
);
