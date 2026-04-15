import type { BBox } from "@/stores/types";

/** Serialize BBox as stable tuple for React Query deep equality */
function bboxKey(bbox: BBox): [number, number, number, number] {
  return [bbox.south, bbox.west, bbox.north, bbox.east];
}

export const queryKeys = {
  health: ["health"] as const,
  areaData: {
    all: ["area-data"] as const,
    bbox: (bbox: BBox, layers: string[]) =>
      ["area-data", ...bboxKey(bbox), [...layers].sort().join(",")] as const,
  },
  score: {
    all: ["score"] as const,
    coord: (lat: number, lng: number, preset?: string) =>
      ["score", lat, lng, preset] as const,
  },
  stats: {
    all: ["stats"] as const,
    bbox: (bbox: BBox) => ["stats", ...bboxKey(bbox)] as const,
  },
  trend: {
    all: ["trend"] as const,
    coord: (lat: number, lng: number, years?: number) =>
      ["trend", lat, lng, years] as const,
  },
  landPrices: {
    all: ["land-prices"] as const,
    byYear: (bbox: BBox, year: number) =>
      ["land-prices", ...bboxKey(bbox), year] as const,
    allYears: (bbox: BBox, fromYear: number, toYear: number) =>
      ["land-prices", "all-years", ...bboxKey(bbox), fromYear, toYear] as const,
    aggregation: (bbox: BBox) =>
      ["land-prices", "aggregation", ...bboxKey(bbox)] as const,
  },
  areaStats: {
    all: ["area-stats"] as const,
    byCode: (code: string) => ["area-stats", code] as const,
  },
  opportunities: {
    all: ["opportunities"] as const,
    list: (
      bbox: BBox,
      filters: {
        tlsMin: number | undefined;
        riskMax: string | undefined;
        zones: string;
        stationMax: number | undefined;
        priceMin: number | undefined;
        priceMax: number | undefined;
        preset: string | undefined;
      },
    ) => ["opportunities", "list", ...bboxKey(bbox), filters] as const,
  },
  transactionSummary: {
    all: ["transaction-summary"] as const,
    byPref: (prefCode: string) => ["transaction-summary", prefCode] as const,
  },
  transactions: {
    all: ["transactions"] as const,
    aggregation: (bbox: BBox) =>
      ["transactions", "aggregation", ...bboxKey(bbox)] as const,
  },
  municipalities: {
    all: ["municipalities"] as const,
    byPref: (prefCode: string) => ["municipalities", prefCode] as const,
  },
};
