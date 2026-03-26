import type { BBox } from "./api";

export const queryKeys = {
  health: ["health"] as const,
  areaData: {
    all: ["area-data"] as const,
    bbox: (bbox: BBox, layers: string[]) =>
      ["area-data", bbox, [...layers].sort().join(",")] as const,
  },
  score: {
    all: ["score"] as const,
    coord: (lat: number, lng: number) => ["score", lat, lng] as const,
  },
  stats: {
    all: ["stats"] as const,
    bbox: (bbox: BBox) => ["stats", bbox] as const,
  },
  trend: {
    all: ["trend"] as const,
    coord: (lat: number, lng: number, years?: number) =>
      ["trend", lat, lng, years] as const,
  },
  landPrices: {
    all: ["land-prices"] as const,
    byYear: (bbox: BBox, year: number) => ["land-prices", bbox, year] as const,
  },
  areaStats: {
    all: ["area-stats"] as const,
    byCode: (code: string) => ["area-stats", code] as const,
  },
};
