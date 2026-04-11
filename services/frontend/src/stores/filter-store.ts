import { create } from "zustand";
import { devtools } from "zustand/middleware";

export type RiskLevel = "low" | "mid" | "high";
export type WeightPreset =
  | "balance"
  | "investment"
  | "residential"
  | "disaster";

export interface FilterState {
  area: {
    prefecture: string;
    cities: string[];
    customPolygon: GeoJSON.Polygon | null;
  };
  criteria: {
    tlsMin: number;
    riskMax: RiskLevel;
    priceRange: [number, number];
  };
  zoning: {
    zones: string[];
    stationMaxDistanceM: number;
  };
  preset: WeightPreset;

  setArea: (a: Partial<FilterState["area"]>) => void;
  setCriteria: (c: Partial<FilterState["criteria"]>) => void;
  setZoning: (z: Partial<FilterState["zoning"]>) => void;
  setPreset: (p: WeightPreset) => void;
  reset: () => void;

  isActive: () => boolean;
  activeCount: () => number;
  toQueryParams: () => Record<string, string>;
}

const DEFAULTS = {
  area: {
    prefecture: "東京都",
    cities: [] as string[],
    customPolygon: null as GeoJSON.Polygon | null,
  },
  criteria: {
    tlsMin: 0,
    riskMax: "high" as RiskLevel,
    priceRange: [0, 10_000_000] as [number, number],
  },
  zoning: {
    zones: [] as string[],
    stationMaxDistanceM: 2000,
  },
  preset: "balance" as WeightPreset,
};

export const useFilterStore = create<FilterState>()(
  devtools(
    (set, get) => ({
      ...DEFAULTS,

      setArea: (a) => set((s) => ({ area: { ...s.area, ...a } })),
      setCriteria: (c) => set((s) => ({ criteria: { ...s.criteria, ...c } })),
      setZoning: (z) => set((s) => ({ zoning: { ...s.zoning, ...z } })),
      setPreset: (p) => set({ preset: p }),
      reset: () =>
        set({
          area: { ...DEFAULTS.area, cities: [], customPolygon: null },
          criteria: { ...DEFAULTS.criteria },
          zoning: { ...DEFAULTS.zoning, zones: [] },
          preset: DEFAULTS.preset,
        }),

      isActive: () => {
        const s = get();
        return (
          s.area.cities.length > 0 ||
          s.area.customPolygon !== null ||
          s.criteria.tlsMin > 0 ||
          s.criteria.riskMax !== "high" ||
          s.criteria.priceRange[0] > 0 ||
          s.criteria.priceRange[1] < 10_000_000 ||
          s.zoning.zones.length > 0 ||
          s.zoning.stationMaxDistanceM < 2000 ||
          s.preset !== "balance"
        );
      },

      activeCount: () => {
        const s = get();
        let n = 0;
        if (s.area.cities.length > 0) n++;
        if (s.area.customPolygon !== null) n++;
        if (s.criteria.tlsMin > 0) n++;
        if (s.criteria.riskMax !== "high") n++;
        if (
          s.criteria.priceRange[0] > 0 ||
          s.criteria.priceRange[1] < 10_000_000
        )
          n++;
        if (s.zoning.zones.length > 0) n++;
        if (s.zoning.stationMaxDistanceM < 2000) n++;
        if (s.preset !== "balance") n++;
        return n;
      },

      toQueryParams: () => {
        const s = get();
        const p: Record<string, string> = {};
        if (s.criteria.tlsMin > 0) p.tls_min = String(s.criteria.tlsMin);
        if (s.criteria.riskMax !== "high") p.risk_max = s.criteria.riskMax;
        if (s.zoning.zones.length > 0) p.zones = s.zoning.zones.join(",");
        if (s.zoning.stationMaxDistanceM < 2000) {
          p.station_max = String(s.zoning.stationMaxDistanceM);
        }
        if (s.criteria.priceRange[0] > 0)
          p.price_min = String(s.criteria.priceRange[0]);
        if (s.criteria.priceRange[1] < 10_000_000)
          p.price_max = String(s.criteria.priceRange[1]);
        if (s.preset !== "balance") p.preset = s.preset;
        if (s.area.cities.length > 0) p.cities = s.area.cities.join(",");
        return p;
      },
    }),
    { name: "filter-store" },
  ),
);
