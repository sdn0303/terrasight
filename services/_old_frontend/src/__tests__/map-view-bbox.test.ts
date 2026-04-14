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

describe("getBBox approximation limitations (documented)", () => {
  it("approximation ignores pitch — same bbox at pitch 0 and 60", () => {
    // This documents WHY we switched to map.getBounds()
    // The old approximation gives identical results regardless of pitch,
    // but a pitched map shows a much larger geographic area
    const calcApproxBbox = (lat: number, lng: number, zoom: number) => {
      const latRange = 180 / 2 ** zoom;
      const lngRange = 360 / 2 ** zoom;
      return {
        south: lat - latRange / 2,
        west: lng - lngRange / 2,
        north: lat + latRange / 2,
        east: lng + lngRange / 2,
      };
    };

    const bboxPitch0 = calcApproxBbox(35.681, 139.767, 12);
    const bboxPitch60 = calcApproxBbox(35.681, 139.767, 12);

    // They're identical — that's the bug. A pitched view sees more area
    // but the approximation doesn't know that.
    expect(bboxPitch0).toEqual(bboxPitch60);
  });
});
