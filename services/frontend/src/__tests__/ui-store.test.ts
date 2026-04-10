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
      addComparePoint({ lat: 35.7, lng: 139.78, address: "C" });
      expect(useUIStore.getState().comparePoints).toHaveLength(3);
    });

    it("ignores 4th point when already at max", () => {
      const { addComparePoint } = useUIStore.getState();
      addComparePoint({ lat: 35.68, lng: 139.76, address: "A" });
      addComparePoint({ lat: 35.69, lng: 139.77, address: "B" });
      addComparePoint({ lat: 35.7, lng: 139.78, address: "C" });
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

describe("ui-store overlay state (Phase 1)", () => {
  beforeEach(() => {
    useUIStore.setState({
      leftPanel: null,
      bottomSheet: null,
      bottomSheetHeightPct: 40,
      insight: null,
      activeTab: "intel",
      settingsOpen: false,
      baseMap: "light",
    });
  });

  it("starts with all overlays closed", () => {
    const s = useUIStore.getState();
    expect(s.leftPanel).toBeNull();
    expect(s.bottomSheet).toBeNull();
    expect(s.insight).toBeNull();
    expect(s.settingsOpen).toBe(false);
  });

  it("starts with bottom sheet height 40%", () => {
    expect(useUIStore.getState().bottomSheetHeightPct).toBe(40);
  });

  it("starts with active tab intel", () => {
    expect(useUIStore.getState().activeTab).toBe("intel");
  });

  it("starts with light base map", () => {
    expect(useUIStore.getState().baseMap).toBe("light");
  });

  it("setLeftPanel sets and unsets the panel", () => {
    const { setLeftPanel } = useUIStore.getState();
    setLeftPanel("finder");
    expect(useUIStore.getState().leftPanel).toBe("finder");
    setLeftPanel(null);
    expect(useUIStore.getState().leftPanel).toBeNull();
  });

  it("toggleLeftPanel opens when closed and closes when same panel is toggled again", () => {
    const { toggleLeftPanel } = useUIStore.getState();
    toggleLeftPanel("finder");
    expect(useUIStore.getState().leftPanel).toBe("finder");
    toggleLeftPanel("finder");
    expect(useUIStore.getState().leftPanel).toBeNull();
  });

  it("toggleLeftPanel switches between different panels (exclusive)", () => {
    const { toggleLeftPanel } = useUIStore.getState();
    toggleLeftPanel("finder");
    expect(useUIStore.getState().leftPanel).toBe("finder");
    toggleLeftPanel("layers");
    expect(useUIStore.getState().leftPanel).toBe("layers");
    toggleLeftPanel("themes");
    expect(useUIStore.getState().leftPanel).toBe("themes");
  });

  it("setBottomSheet opens and closes opportunities", () => {
    const { setBottomSheet } = useUIStore.getState();
    setBottomSheet("opportunities");
    expect(useUIStore.getState().bottomSheet).toBe("opportunities");
    setBottomSheet(null);
    expect(useUIStore.getState().bottomSheet).toBeNull();
  });

  it("setBottomSheetHeightPct clamps to 20-80 range", () => {
    const { setBottomSheetHeightPct } = useUIStore.getState();
    setBottomSheetHeightPct(10);
    expect(useUIStore.getState().bottomSheetHeightPct).toBe(20);
    setBottomSheetHeightPct(90);
    expect(useUIStore.getState().bottomSheetHeightPct).toBe(80);
    setBottomSheetHeightPct(55);
    expect(useUIStore.getState().bottomSheetHeightPct).toBe(55);
  });

  it("setInsight with point context", () => {
    const { setInsight } = useUIStore.getState();
    setInsight({ kind: "point", lat: 35.6595, lng: 139.7004 });
    const insight = useUIStore.getState().insight;
    expect(insight).not.toBeNull();
    if (insight && insight.kind === "point") {
      expect(insight.lat).toBe(35.6595);
      expect(insight.lng).toBe(139.7004);
    }
  });

  it("setInsight with property context", () => {
    const { setInsight } = useUIStore.getState();
    setInsight({ kind: "property", id: "lp_123", lat: 35.6, lng: 139.7 });
    const insight = useUIStore.getState().insight;
    expect(insight).not.toBeNull();
    if (insight && insight.kind === "property") {
      expect(insight.id).toBe("lp_123");
    }
  });

  it("setActiveTab switches drawer tabs", () => {
    const { setActiveTab } = useUIStore.getState();
    setActiveTab("trend");
    expect(useUIStore.getState().activeTab).toBe("trend");
    setActiveTab("risk");
    expect(useUIStore.getState().activeTab).toBe("risk");
  });

  it("setBaseMap switches base map", () => {
    const { setBaseMap } = useUIStore.getState();
    setBaseMap("dark");
    expect(useUIStore.getState().baseMap).toBe("dark");
    setBaseMap("satellite");
    expect(useUIStore.getState().baseMap).toBe("satellite");
  });

  it("setSettingsOpen toggles settings modal", () => {
    const { setSettingsOpen } = useUIStore.getState();
    setSettingsOpen(true);
    expect(useUIStore.getState().settingsOpen).toBe(true);
    setSettingsOpen(false);
    expect(useUIStore.getState().settingsOpen).toBe(false);
  });

  it("legacy mode field is still present and functional", () => {
    const { setMode } = useUIStore.getState();
    setMode("compare");
    expect(useUIStore.getState().mode).toBe("compare");
    setMode("explore");
    expect(useUIStore.getState().mode).toBe("explore");
  });

  it("legacy layerSettingsOpen is still present and toggleable", () => {
    const { toggleLayerSettings } = useUIStore.getState();
    const before = useUIStore.getState().layerSettingsOpen;
    toggleLayerSettings();
    expect(useUIStore.getState().layerSettingsOpen).toBe(!before);
  });
});
