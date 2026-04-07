import { beforeEach, describe, expect, it, vi } from "vitest";
import { getLayerIdsByTheme, getLayerIdsForThemes } from "@/lib/themes";
import { LAYERS } from "@/lib/layers";
import { useMapStore } from "@/stores/map-store";
import { useUIStore } from "@/stores/ui-store";

vi.mock("next-intl", () => ({
  useTranslations: () => (key: string) => key,
}));

const defaultLayerIds = new Set(
  LAYERS.filter((l) => l.defaultEnabled).map((l) => l.id),
);

describe("Theme layer counts", () => {
  it("safety theme has layers", () => {
    const layers = getLayerIdsByTheme("safety");
    expect(layers.length).toBeGreaterThan(0);
  });

  it("all four themes have at least one layer", () => {
    for (const id of ["safety", "livability", "price", "future"] as const) {
      expect(getLayerIdsByTheme(id).length).toBeGreaterThan(0);
    }
  });

  it("getLayerIdsForThemes always includes admin_boundary", () => {
    const ids = getLayerIdsForThemes(new Set(["safety"]));
    expect(ids.has("admin_boundary")).toBe(true);
  });
});

describe("Theme-store integration", () => {
  beforeEach(() => {
    useMapStore.setState({ visibleLayers: new Set(defaultLayerIds) });
    useUIStore.setState({ activeThemes: new Set() });
  });

  it("applying safety theme adds safety layers and preserves defaults", () => {
    const safetyLayers = getLayerIdsForThemes(new Set(["safety"]));
    useMapStore.getState().applyThemeLayers(safetyLayers);
    const visible = useMapStore.getState().visibleLayers;
    for (const id of safetyLayers) {
      expect(visible.has(id)).toBe(true);
    }
    for (const id of defaultLayerIds) {
      expect(visible.has(id)).toBe(true);
    }
  });

  it("clearing themes restores defaults without wiping everything", () => {
    const safetyLayers = getLayerIdsForThemes(new Set(["safety"]));
    useMapStore.getState().applyThemeLayers(safetyLayers);
    useMapStore.getState().applyThemeLayers(new Set());
    const visible = useMapStore.getState().visibleLayers;
    for (const id of defaultLayerIds) {
      expect(visible.has(id)).toBe(true);
    }
    expect(visible.has("flood")).toBe(false);
    expect(visible.has("steep_slope")).toBe(false);
  });
});
