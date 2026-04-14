import { beforeEach, describe, expect, it } from "vitest";
import { LAYERS } from "@/lib/layers";
import { useMapStore } from "@/stores/map-store";

const realDefaultLayers = new Set(
  LAYERS.filter((l) => l.defaultEnabled).map((l) => l.id),
);

describe("useMapStore", () => {
  beforeEach(() => {
    useMapStore.setState({
      visibleLayers: new Set(realDefaultLayers),
      selectedFeature: null,
    });
  });

  it("toggles layer visibility", () => {
    useMapStore.getState().toggleLayer("flood");
    expect(useMapStore.getState().visibleLayers.has("flood")).toBe(true);

    useMapStore.getState().toggleLayer("flood");
    expect(useMapStore.getState().visibleLayers.has("flood")).toBe(false);
  });

  it("selects and deselects feature", () => {
    const feature = {
      layerId: "landprice",
      properties: { id: 1, price_per_sqm: 1200000 },
      coordinates: [139.767, 35.681] as [number, number],
    };
    useMapStore.getState().selectFeature(feature);
    expect(useMapStore.getState().selectedFeature).toEqual(feature);

    useMapStore.getState().selectFeature(null);
    expect(useMapStore.getState().selectedFeature).toBeNull();
  });

  it("default visible layers match LAYERS defaultEnabled", () => {
    const layers = useMapStore.getState().visibleLayers;
    for (const id of realDefaultLayers) {
      expect(layers.has(id)).toBe(true);
    }
    expect(layers.has("flood")).toBe(false);
    expect(layers.has("landprice")).toBe(false);
  });

  describe("applyThemeLayers", () => {
    it("unions theme layers with defaults", () => {
      const themeLayers = new Set(["flood", "steep_slope"]);
      useMapStore.getState().applyThemeLayers(themeLayers);

      const layers = useMapStore.getState().visibleLayers;
      // theme layers are added
      expect(layers.has("flood")).toBe(true);
      expect(layers.has("steep_slope")).toBe(true);
      // defaults are preserved
      for (const id of realDefaultLayers) {
        expect(layers.has(id)).toBe(true);
      }
    });

    it("with empty set restores defaults only", () => {
      // start with something extra toggled
      useMapStore.getState().toggleLayer("flood");
      useMapStore.getState().applyThemeLayers(new Set());

      const layers = useMapStore.getState().visibleLayers;
      expect(layers.has("flood")).toBe(false);
      for (const id of realDefaultLayers) {
        expect(layers.has(id)).toBe(true);
      }
    });

    it("preserves manually toggled layers", () => {
      // manually add a layer before applying theme
      useMapStore.getState().toggleLayer("station");
      const themeLayers = new Set(["flood"]);
      useMapStore.getState().applyThemeLayers(themeLayers);

      const layers = useMapStore.getState().visibleLayers;
      // theme layer added
      expect(layers.has("flood")).toBe(true);
      // manually toggled layer preserved
      expect(layers.has("station")).toBe(true);
      // defaults preserved
      for (const id of realDefaultLayers) {
        expect(layers.has(id)).toBe(true);
      }
    });
  });

  describe("resetToDefaults", () => {
    it("restores only default layers", () => {
      // add some extra layers
      useMapStore.getState().toggleLayer("flood");
      useMapStore.getState().toggleLayer("station");
      useMapStore.getState().resetToDefaults();

      const layers = useMapStore.getState().visibleLayers;
      expect(layers.has("flood")).toBe(false);
      expect(layers.has("station")).toBe(false);
      for (const id of realDefaultLayers) {
        expect(layers.has(id)).toBe(true);
      }
      expect(layers.size).toBe(realDefaultLayers.size);
    });
  });
});
