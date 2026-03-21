import { beforeEach, describe, expect, it } from "vitest";
import { useUIStore } from "@/stores/ui-store";

describe("useUIStore", () => {
  beforeEach(() => {
    useUIStore.setState({
      compareMode: false,
      comparePointA: null,
      comparePointB: null,
      layerPanelOpen: true,
    });
  });

  it("starts with compare mode off", () => {
    expect(useUIStore.getState().compareMode).toBe(false);
  });

  it("starts with layer panel open", () => {
    expect(useUIStore.getState().layerPanelOpen).toBe(true);
  });

  it("enters compare mode and resets points", () => {
    useUIStore.getState().setComparePoint({
      lat: 35.681,
      lng: 139.767,
      address: "東京駅",
    });
    useUIStore.getState().enterCompareMode();

    const state = useUIStore.getState();
    expect(state.compareMode).toBe(true);
    expect(state.comparePointA).toBeNull();
    expect(state.comparePointB).toBeNull();
  });

  it("exits compare mode and resets points", () => {
    useUIStore.getState().enterCompareMode();
    useUIStore.getState().setComparePoint({
      lat: 35.681,
      lng: 139.767,
      address: "東京駅",
    });
    useUIStore.getState().exitCompareMode();

    const state = useUIStore.getState();
    expect(state.compareMode).toBe(false);
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

  it("toggles layer panel", () => {
    expect(useUIStore.getState().layerPanelOpen).toBe(true);
    useUIStore.getState().toggleLayerPanel();
    expect(useUIStore.getState().layerPanelOpen).toBe(false);
    useUIStore.getState().toggleLayerPanel();
    expect(useUIStore.getState().layerPanelOpen).toBe(true);
  });
});
