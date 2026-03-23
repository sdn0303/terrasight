import { describe, expect, it } from "vitest";
import { queryKeys } from "@/lib/query-keys";

describe("queryKeys", () => {
  it("health key is a static array", () => {
    expect(queryKeys.health).toEqual(["health"]);
  });

  it("areaData.bbox produces deterministic key regardless of layer order", () => {
    const bbox = { south: 35.6, west: 139.7, north: 35.8, east: 139.9 };
    const key1 = queryKeys.areaData.bbox(bbox, ["flood", "landprice"]);
    const key2 = queryKeys.areaData.bbox(bbox, ["landprice", "flood"]);
    // Layers are sorted internally, so keys should match
    expect(key1).toEqual(key2);
  });

  it("areaData.bbox includes bbox in key for cache isolation", () => {
    const bbox1 = { south: 35.6, west: 139.7, north: 35.8, east: 139.9 };
    const bbox2 = { south: 35.0, west: 139.0, north: 35.2, east: 139.2 };
    const key1 = queryKeys.areaData.bbox(bbox1, ["landprice"]);
    const key2 = queryKeys.areaData.bbox(bbox2, ["landprice"]);
    expect(key1).not.toEqual(key2);
  });

  it("score.coord differentiates by coordinates", () => {
    const key1 = queryKeys.score.coord(35.681, 139.767);
    const key2 = queryKeys.score.coord(35.682, 139.767);
    expect(key1).not.toEqual(key2);
    expect(key1).toEqual(["score", 35.681, 139.767]);
  });

  it("stats.bbox includes the bbox object", () => {
    const bbox = { south: 35.6, west: 139.7, north: 35.8, east: 139.9 };
    expect(queryKeys.stats.bbox(bbox)).toEqual(["stats", bbox]);
  });

  it("trend.coord includes optional years parameter", () => {
    const withYears = queryKeys.trend.coord(35.681, 139.767, 10);
    const withoutYears = queryKeys.trend.coord(35.681, 139.767);
    expect(withYears).toEqual(["trend", 35.681, 139.767, 10]);
    expect(withoutYears).toEqual(["trend", 35.681, 139.767, undefined]);
    expect(withYears).not.toEqual(withoutYears);
  });
});
