import { beforeEach, describe, expect, it } from "vitest";
import { useUIStore } from "@/stores/ui-store";

describe("useUIStore", () => {
  beforeEach(() => {
    useUIStore.setState({
      mode: "explore",
      comparePointA: null,
      comparePointB: null,
      layerSettingsOpen: true,
    });
  });

  it("starts with explore mode", () => {
    expect(useUIStore.getState().mode).toBe("explore");
  });

  it("starts with layer settings open", () => {
    expect(useUIStore.getState().layerSettingsOpen).toBe(true);
  });

  it("enters compare mode via setMode and resets points", () => {
    useUIStore.getState().setComparePoint({
      lat: 35.681,
      lng: 139.767,
      address: "東京駅",
    });
    useUIStore.getState().resetCompare();
    useUIStore.getState().setMode("compare");

    const state = useUIStore.getState();
    expect(state.mode).toBe("compare");
    expect(state.comparePointA).toBeNull();
    expect(state.comparePointB).toBeNull();
  });

  it("exits compare mode by resetCompare + setMode explore", () => {
    useUIStore.getState().setMode("compare");
    useUIStore.getState().setComparePoint({
      lat: 35.681,
      lng: 139.767,
      address: "東京駅",
    });
    useUIStore.getState().resetCompare();
    useUIStore.getState().setMode("explore");

    const state = useUIStore.getState();
    expect(state.mode).toBe("explore");
    expect(state.comparePointA).toBeNull();
    expect(state.comparePointB).toBeNull();
  });

  it("sets compare point A first, then B", () => {
    const pointA = { lat: 35.681, lng: 139.767, address: "東京駅" };
    const pointB = { lat: 35.690, lng: 139.700, address: "新宿駅" };

    useUIStore.getState().setComparePoint(pointA);
    expect(useUIStore.getState().comparePointA).toEqual(pointA);
    expect(useUIStore.getState().comparePointB).toBeNull();

    useUIStore.getState().setComparePoint(pointB);
    expect(useUIStore.getState().comparePointA).toEqual(pointA);
    expect(useUIStore.getState().comparePointB).toEqual(pointB);
  });

  it("toggles layer settings", () => {
    expect(useUIStore.getState().layerSettingsOpen).toBe(true);
    useUIStore.getState().toggleLayerSettings();
    expect(useUIStore.getState().layerSettingsOpen).toBe(false);
    useUIStore.getState().toggleLayerSettings();
    expect(useUIStore.getState().layerSettingsOpen).toBe(true);
  });
});
