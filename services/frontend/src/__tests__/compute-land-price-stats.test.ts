import type { FeatureCollection, Feature, Point } from "geojson";
import { describe, expect, it } from "vitest";
import {
  computeLandPriceStats,
  type LandPriceStats,
} from "@/features/stats/utils/compute-land-price-stats";

// ─── Helpers ─────────────────────────────────────────

function makePoint(price_per_sqm: number | null | undefined): Feature<Point> {
  return {
    type: "Feature",
    geometry: { type: "Point", coordinates: [139.7, 35.6] },
    properties: price_per_sqm !== undefined ? { price_per_sqm } : {},
  };
}

function makeFC(features: Feature<Point>[]): FeatureCollection {
  return { type: "FeatureCollection", features };
}

// ─── Tests ───────────────────────────────────────────

describe("computeLandPriceStats", () => {
  it("returns zeros for an empty FeatureCollection", () => {
    const result = computeLandPriceStats(makeFC([]));
    const expected: LandPriceStats = {
      avg_per_sqm: 0,
      median_per_sqm: 0,
      min_per_sqm: 0,
      max_per_sqm: 0,
      count: 0,
    };
    expect(result).toEqual(expected);
  });

  it("returns zeros when fc is undefined", () => {
    const result = computeLandPriceStats(undefined);
    expect(result.count).toBe(0);
    expect(result.avg_per_sqm).toBe(0);
  });

  it("computes correct stats for a single feature", () => {
    const result = computeLandPriceStats(makeFC([makePoint(500_000)]));
    expect(result.count).toBe(1);
    expect(result.avg_per_sqm).toBe(500_000);
    expect(result.median_per_sqm).toBe(500_000);
    expect(result.min_per_sqm).toBe(500_000);
    expect(result.max_per_sqm).toBe(500_000);
  });

  it("computes correct avg/median/min/max for multiple features (odd count)", () => {
    // Prices: 100_000, 200_000, 900_000  → sorted: [100k, 200k, 900k]
    const fc = makeFC([
      makePoint(900_000),
      makePoint(100_000),
      makePoint(200_000),
    ]);
    const result = computeLandPriceStats(fc);
    expect(result.count).toBe(3);
    expect(result.min_per_sqm).toBe(100_000);
    expect(result.max_per_sqm).toBe(900_000);
    expect(result.avg_per_sqm).toBe(Math.round((100_000 + 200_000 + 900_000) / 3));
    // Median of [100k, 200k, 900k] is 200k
    expect(result.median_per_sqm).toBe(200_000);
  });

  it("computes correct median for even count", () => {
    // Prices: 100_000, 200_000, 300_000, 400_000
    const fc = makeFC([
      makePoint(300_000),
      makePoint(100_000),
      makePoint(400_000),
      makePoint(200_000),
    ]);
    const result = computeLandPriceStats(fc);
    expect(result.count).toBe(4);
    // Median = (200k + 300k) / 2 = 250k
    expect(result.median_per_sqm).toBe(250_000);
    expect(result.min_per_sqm).toBe(100_000);
    expect(result.max_per_sqm).toBe(400_000);
  });

  it("filters out features with missing price_per_sqm", () => {
    const fc = makeFC([
      makePoint(500_000),
      makePoint(undefined), // missing property entirely
      makePoint(300_000),
    ]);
    const result = computeLandPriceStats(fc);
    expect(result.count).toBe(2);
    expect(result.min_per_sqm).toBe(300_000);
    expect(result.max_per_sqm).toBe(500_000);
  });

  it("filters out features with null price_per_sqm", () => {
    const fc = makeFC([
      makePoint(null),
      makePoint(800_000),
    ]);
    const result = computeLandPriceStats(fc);
    expect(result.count).toBe(1);
    expect(result.avg_per_sqm).toBe(800_000);
  });

  it("filters out features with non-positive price_per_sqm", () => {
    const fc = makeFC([
      makePoint(0),
      makePoint(-50_000),
      makePoint(600_000),
    ]);
    const result = computeLandPriceStats(fc);
    // Only 600_000 passes the p > 0 filter
    expect(result.count).toBe(1);
    expect(result.avg_per_sqm).toBe(600_000);
  });

  it("returns zeros when all features have invalid prices", () => {
    const fc = makeFC([makePoint(null), makePoint(0), makePoint(-1)]);
    const result = computeLandPriceStats(fc);
    expect(result).toEqual({
      avg_per_sqm: 0,
      median_per_sqm: 0,
      min_per_sqm: 0,
      max_per_sqm: 0,
      count: 0,
    });
  });
});
