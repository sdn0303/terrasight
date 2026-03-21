import { describe, expect, it, beforeEach } from "vitest";
import { useMapStore } from "@/stores/map-store";

describe("useMapStore", () => {
  beforeEach(() => {
    useMapStore.setState({
      visibleLayers: new Set(["landprice", "zoning"]),
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

  it("default visible layers are landprice and zoning", () => {
    useMapStore.setState({ visibleLayers: new Set(["landprice", "zoning"]) });
    const layers = useMapStore.getState().visibleLayers;
    expect(layers.has("landprice")).toBe(true);
    expect(layers.has("zoning")).toBe(true);
    expect(layers.has("flood")).toBe(false);
  });
});
