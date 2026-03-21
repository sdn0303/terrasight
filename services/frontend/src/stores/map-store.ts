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

interface MapState {
  viewState: ViewState;
  visibleLayers: Set<string>;
  selectedFeature: SelectedFeature | null;
  setViewState: (viewState: ViewState) => void;
  toggleLayer: (layerId: string) => void;
  selectFeature: (feature: SelectedFeature | null) => void;
  getBBox: () => { south: number; west: number; north: number; east: number };
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

      selectFeature: (feature) => set({ selectedFeature: feature }),

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
