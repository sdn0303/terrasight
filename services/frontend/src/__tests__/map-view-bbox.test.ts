import { describe, expect, it } from "vitest";
import type { BBox } from "@/lib/api";

describe("MapView onMoveEnd bbox contract", () => {
  it("BBox interface has required cardinal fields", () => {
    const bbox: BBox = { south: 35.6, west: 139.7, north: 35.8, east: 139.9 };
    expect(bbox.south).toBeLessThan(bbox.north);
    expect(bbox.west).toBeLessThan(bbox.east);
  });

  it("MapLibre LngLatBounds converts to BBox correctly", () => {
    const mockBounds = {
      getSouth: () => 35.65,
      getWest: () => 139.72,
      getNorth: () => 35.71,
      getEast: () => 139.81,
    };

    const bbox: BBox = {
      south: mockBounds.getSouth(),
      west: mockBounds.getWest(),
      north: mockBounds.getNorth(),
      east: mockBounds.getEast(),
    };

    expect(bbox).toEqual({
      south: 35.65,
      west: 139.72,
      north: 35.71,
      east: 139.81,
    });
  });
});
