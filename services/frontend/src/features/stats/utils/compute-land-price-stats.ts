import type { FeatureCollection } from "geojson";

export interface LandPriceStats {
  avg_per_sqm: number;
  median_per_sqm: number;
  min_per_sqm: number;
  max_per_sqm: number;
  count: number;
}

export function computeLandPriceStats(
  fc: FeatureCollection | undefined,
): LandPriceStats {
  const empty: LandPriceStats = {
    avg_per_sqm: 0,
    median_per_sqm: 0,
    min_per_sqm: 0,
    max_per_sqm: 0,
    count: 0,
  };
  if (!fc || fc.features.length === 0) return empty;

  const prices = fc.features
    .map((f) => {
      const props = f.properties as Record<string, unknown> | null;
      return props?.price_per_sqm;
    })
    .filter((p): p is number => typeof p === "number" && p > 0)
    .sort((a, b) => a - b);

  if (prices.length === 0) return empty;

  const sum = prices.reduce((s, p) => s + p, 0);
  const mid = Math.floor(prices.length / 2);

  const midLow = prices[mid - 1];
  const midVal = prices[mid];
  if (midVal === undefined) return empty;

  const median =
    prices.length % 2 === 0 ? ((midLow ?? midVal) + midVal) / 2 : midVal;

  const minVal = prices[0];
  const maxVal = prices[prices.length - 1];
  if (minVal === undefined || maxVal === undefined) return empty;

  return {
    avg_per_sqm: Math.round(sum / prices.length),
    median_per_sqm: Math.round(median),
    min_per_sqm: minVal,
    max_per_sqm: maxVal,
    count: prices.length,
  };
}
