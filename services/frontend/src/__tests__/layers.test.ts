import { describe, expect, it } from "vitest";
import {
  ALL_INTERACTIVE_LAYER_IDS,
  CATEGORIES,
  getLayersBySource,
  LAYERS,
} from "@/lib/layers";

describe("LAYERS configuration", () => {
  it("has 21 layers total", () => {
    expect(LAYERS).toHaveLength(21);
  });

  it("every layer has required fields", () => {
    for (const layer of LAYERS) {
      expect(layer.id, `${layer.id}: id`).toBeTruthy();
      expect(layer.name, `${layer.id}: name`).toBeTruthy();
      expect(layer.nameJa, `${layer.id}: nameJa`).toBeTruthy();
      expect(layer.color, `${layer.id}: color`).toBeTruthy();
      expect(["api", "static", "timeseries"]).toContain(layer.source);
      expect(["value", "risk", "ground", "infra", "orientation"]).toContain(
        layer.category,
      );
    }
  });

  it("has unique layer IDs", () => {
    const ids = LAYERS.map((l) => l.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("every layer has at least one interactiveLayerId", () => {
    for (const layer of LAYERS) {
      expect(
        layer.interactiveLayerIds?.length,
        `${layer.id}: missing interactiveLayerIds`,
      ).toBeGreaterThan(0);
    }
  });

  it("interactiveLayerIds are globally unique", () => {
    const allIds = LAYERS.flatMap((l) => l.interactiveLayerIds ?? []);
    expect(new Set(allIds).size).toBe(allIds.length);
  });

  it("popupFields exist for all layers", () => {
    for (const layer of LAYERS) {
      expect(
        layer.popupFields?.length,
        `${layer.id}: missing popupFields`,
      ).toBeGreaterThan(0);
    }
  });

  it("popupField keys are unique within each layer", () => {
    for (const layer of LAYERS) {
      const keys = (layer.popupFields ?? []).map((f) => f.key);
      expect(new Set(keys).size, `${layer.id}: duplicate popupField keys`).toBe(
        keys.length,
      );
    }
  });

  it("default enabled layers include land_price_ts and zoning (landprice off)", () => {
    const defaults = LAYERS.filter((l) => l.defaultEnabled).map((l) => l.id);
    expect(defaults).toContain("land_price_ts");
    expect(defaults).not.toContain("landprice");
    expect(defaults).toContain("zoning");
  });

  it("minZoom is only set for high-detail layers", () => {
    const withMinZoom = LAYERS.filter((l) => l.minZoom !== undefined);
    for (const layer of withMinZoom) {
      expect(layer.minZoom, `${layer.id}: minZoom`).toBeGreaterThanOrEqual(10);
      expect(layer.minZoom, `${layer.id}: minZoom`).toBeLessThanOrEqual(15);
    }
  });
});

describe("CATEGORIES", () => {
  it("has 5 categories", () => {
    expect(CATEGORIES).toHaveLength(5);
  });

  it("every layer belongs to a valid category", () => {
    const categoryIds = new Set(CATEGORIES.map((c) => c.id));
    for (const layer of LAYERS) {
      expect(
        categoryIds.has(layer.category),
        `${layer.id}: invalid category '${layer.category}'`,
      ).toBe(true);
    }
  });

  it("every category has at least one layer", () => {
    for (const category of CATEGORIES) {
      const count = LAYERS.filter((l) => l.category === category.id).length;
      expect(count, `${category.id}: no layers`).toBeGreaterThan(0);
    }
  });
});

describe("ALL_INTERACTIVE_LAYER_IDS", () => {
  it("has correct count derived from LAYERS", () => {
    const expected = LAYERS.flatMap((l) => l.interactiveLayerIds ?? []);
    expect(ALL_INTERACTIVE_LAYER_IDS).toEqual(expected);
  });

  it("contains no duplicates", () => {
    expect(new Set(ALL_INTERACTIVE_LAYER_IDS).size).toBe(
      ALL_INTERACTIVE_LAYER_IDS.length,
    );
  });
});

describe("getLayersBySource", () => {
  it("returns only API layers", () => {
    const apiLayers = getLayersBySource("api");
    expect(apiLayers.length).toBeGreaterThan(0);
    for (const layer of apiLayers) {
      expect(layer.source).toBe("api");
    }
  });

  it("returns only static layers", () => {
    const staticLayers = getLayersBySource("static");
    expect(staticLayers.length).toBeGreaterThan(0);
    for (const layer of staticLayers) {
      expect(layer.source).toBe("static");
    }
  });

  it("API + static + timeseries = total layers", () => {
    const api = getLayersBySource("api");
    const staticL = getLayersBySource("static");
    const timeseries = getLayersBySource("timeseries");
    expect(api.length + staticL.length + timeseries.length).toBe(LAYERS.length);
  });
});

describe("PR1 new layers", () => {
  const newLayerIds = ["station", "landslide", "population_mesh"];

  it("all 3 new layers exist in LAYERS", () => {
    const existingIds = new Set(LAYERS.map((l) => l.id));
    for (const id of newLayerIds) {
      expect(existingIds.has(id), `missing layer: ${id}`).toBe(true);
    }
  });

  it("all 4 new layers are static source", () => {
    for (const id of newLayerIds) {
      const layer = LAYERS.find((l) => l.id === id);
      expect(layer?.source, `${id}: not static`).toBe("static");
    }
  });

  it("population_mesh has correct category", () => {
    const mesh = LAYERS.find((l) => l.id === "population_mesh");
    expect(mesh?.category).toBe("orientation");
    expect(mesh?.minZoom).toBe(13);
  });

  it("landslide has minZoom 11", () => {
    const landslide = LAYERS.find((l) => l.id === "landslide");
    expect(landslide?.minZoom).toBe(11);
  });
});
