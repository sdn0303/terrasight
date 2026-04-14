import { beforeEach, describe, expect, it } from "vitest";
import { SpatialEngineAdapter } from "@/lib/wasm/spatial-engine";

describe("SpatialEngineAdapter ready state", () => {
  let adapter: SpatialEngineAdapter;

  beforeEach(() => {
    adapter = new SpatialEngineAdapter();
  });

  it("starts with empty loadedLayers", () => {
    expect(adapter.loadedLayers.size).toBe(0);
  });

  it("queryReady returns false when no layers loaded", () => {
    expect(adapter.queryReady(["geology"])).toBe(false);
  });

  it("queryReady returns true after layer is registered", () => {
    adapter.registerLoadedLayers({ geology: 133, landform: 370 });
    expect(adapter.queryReady(["geology"])).toBe(true);
    expect(adapter.queryReady(["geology", "landform"])).toBe(true);
  });

  it("queryReady returns false if any requested layer is missing", () => {
    adapter.registerLoadedLayers({ geology: 133 });
    expect(adapter.queryReady(["geology", "missing"])).toBe(false);
  });

  it("queryReady normalizes underscore IDs to canonical form", () => {
    adapter.registerLoadedLayers({ "admin-boundary": 6902 });
    expect(adapter.queryReady(["admin_boundary"])).toBe(true);
  });

  it("registerLoadedLayers normalizes keys to canonical form", () => {
    adapter.registerLoadedLayers({ admin_boundary: 6902 });
    expect(adapter.loadedLayers.has("admin-boundary")).toBe(true);
    expect(adapter.loadedLayers.has("admin_boundary")).toBe(false);
  });

  it("ready is false when no layers loaded", () => {
    expect(adapter.ready).toBe(false);
  });
});
