import { beforeEach, describe, expect, it } from "vitest";
import { useUIStore } from "@/stores/ui-store";

describe("useUIStore", () => {
  beforeEach(() => {
    useUIStore.setState({
      mode: "explore",
      comparePoints: [],
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
    useUIStore.getState().addComparePoint({
      lat: 35.681,
      lng: 139.767,
      address: "東京駅",
    });
    useUIStore.getState().resetCompare();
    useUIStore.getState().setMode("compare");

    const state = useUIStore.getState();
    expect(state.mode).toBe("compare");
    expect(state.comparePoints).toHaveLength(0);
  });

  it("exits compare mode by resetCompare + setMode explore", () => {
    useUIStore.getState().setMode("compare");
    useUIStore.getState().addComparePoint({
      lat: 35.681,
      lng: 139.767,
      address: "東京駅",
    });
    useUIStore.getState().resetCompare();
    useUIStore.getState().setMode("explore");

    const state = useUIStore.getState();
    expect(state.mode).toBe("explore");
    expect(state.comparePoints).toHaveLength(0);
  });

  it("toggles layer settings", () => {
    expect(useUIStore.getState().layerSettingsOpen).toBe(true);
    useUIStore.getState().toggleLayerSettings();
    expect(useUIStore.getState().layerSettingsOpen).toBe(false);
    useUIStore.getState().toggleLayerSettings();
    expect(useUIStore.getState().layerSettingsOpen).toBe(true);
  });

  describe("N-point compare", () => {
    beforeEach(() => {
      useUIStore.getState().resetCompare();
    });

    it("adds up to 3 compare points", () => {
      const { addComparePoint } = useUIStore.getState();
      addComparePoint({ lat: 35.68, lng: 139.76, address: "A" });
      addComparePoint({ lat: 35.69, lng: 139.77, address: "B" });
      addComparePoint({ lat: 35.70, lng: 139.78, address: "C" });
      expect(useUIStore.getState().comparePoints).toHaveLength(3);
    });

    it("ignores 4th point when already at max", () => {
      const { addComparePoint } = useUIStore.getState();
      addComparePoint({ lat: 35.68, lng: 139.76, address: "A" });
      addComparePoint({ lat: 35.69, lng: 139.77, address: "B" });
      addComparePoint({ lat: 35.70, lng: 139.78, address: "C" });
      addComparePoint({ lat: 35.71, lng: 139.79, address: "D" });
      expect(useUIStore.getState().comparePoints).toHaveLength(3);
    });

    it("removes compare point by index", () => {
      const { addComparePoint } = useUIStore.getState();
      addComparePoint({ lat: 35.68, lng: 139.76, address: "A" });
      addComparePoint({ lat: 35.69, lng: 139.77, address: "B" });
      useUIStore.getState().removeComparePoint(0);
      const pts = useUIStore.getState().comparePoints;
      expect(pts).toHaveLength(1);
      expect(pts[0]?.address).toBe("B");
    });

    it("resetCompare clears all points", () => {
      const { addComparePoint } = useUIStore.getState();
      addComparePoint({ lat: 35.68, lng: 139.76, address: "A" });
      addComparePoint({ lat: 35.69, lng: 139.77, address: "B" });
      useUIStore.getState().resetCompare();
      expect(useUIStore.getState().comparePoints).toHaveLength(0);
    });
  });
});
