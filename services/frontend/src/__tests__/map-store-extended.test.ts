import { beforeEach, describe, expect, it } from "vitest";
import { LAYERS } from "@/lib/layers";
import { useMapStore } from "@/stores/map-store";

describe("useMapStore — extended tests", () => {
  beforeEach(() => {
    // Reset to defaults
    const defaultVisibleLayers = new Set(
      LAYERS.filter((l) => l.defaultEnabled).map((l) => l.id),
    );
    useMapStore.setState({
      viewState: {
        latitude: 35.681,
        longitude: 139.767,
        zoom: 12,
        pitch: 45,
        bearing: 0,
      },
      visibleLayers: defaultVisibleLayers,
      selectedFeature: null,
    });
  });

  it("default visible layers match LAYERS config", () => {
    const expected = new Set(
      LAYERS.filter((l) => l.defaultEnabled).map((l) => l.id),
    );
    const actual = useMapStore.getState().visibleLayers;
    expect(actual).toEqual(expected);
  });

  it("toggleLayer adds new layer to set", () => {
    useMapStore.getState().toggleLayer("population_mesh");
    expect(
      useMapStore.getState().visibleLayers.has("population_mesh"),
    ).toBe(true);
  });

  it("toggleLayer is idempotent — double toggle returns to original", () => {
    const before = new Set(useMapStore.getState().visibleLayers);
    useMapStore.getState().toggleLayer("flood");
    useMapStore.getState().toggleLayer("flood");
    expect(useMapStore.getState().visibleLayers).toEqual(before);
  });

  it("toggleLayer does not mutate previous set reference", () => {
    const before = useMapStore.getState().visibleLayers;
    useMapStore.getState().toggleLayer("flood");
    const after = useMapStore.getState().visibleLayers;
    expect(before).not.toBe(after); // Different reference
  });

  it("getBBox returns correct bounds for default viewState", () => {
    const bbox = useMapStore.getState().getBBox();
    expect(bbox.south).toBeLessThan(bbox.north);
    expect(bbox.west).toBeLessThan(bbox.east);
    // Center should be approximately between bounds
    expect(bbox.south).toBeLessThan(35.681);
    expect(bbox.north).toBeGreaterThan(35.681);
    expect(bbox.west).toBeLessThan(139.767);
    expect(bbox.east).toBeGreaterThan(139.767);
  });

  it("getBBox shrinks as zoom increases", () => {
    useMapStore.getState().setViewState({
      latitude: 35.681,
      longitude: 139.767,
      zoom: 12,
      pitch: 0,
      bearing: 0,
    });
    const bboxZ12 = useMapStore.getState().getBBox();
    const rangeZ12 = bboxZ12.north - bboxZ12.south;

    useMapStore.getState().setViewState({
      latitude: 35.681,
      longitude: 139.767,
      zoom: 15,
      pitch: 0,
      bearing: 0,
    });
    const bboxZ15 = useMapStore.getState().getBBox();
    const rangeZ15 = bboxZ15.north - bboxZ15.south;

    expect(rangeZ15).toBeLessThan(rangeZ12);
  });

  it("selectFeature stores coordinates as [lng, lat]", () => {
    const feature = {
      layerId: "station-circle",
      properties: { stationName: "東京", lineName: "中央線" },
      coordinates: [139.767, 35.681] as [number, number],
    };
    useMapStore.getState().selectFeature(feature);
    const selected = useMapStore.getState().selectedFeature;
    expect(selected?.coordinates[0]).toBe(139.767); // lng
    expect(selected?.coordinates[1]).toBe(35.681); // lat
  });

  it("selectFeature(null) clears selection", () => {
    useMapStore.getState().selectFeature({
      layerId: "flood-fill",
      properties: {},
      coordinates: [139.0, 35.0],
    });
    expect(useMapStore.getState().selectedFeature).not.toBeNull();

    useMapStore.getState().selectFeature(null);
    expect(useMapStore.getState().selectedFeature).toBeNull();
  });
});
