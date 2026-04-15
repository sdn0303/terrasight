import { create } from "zustand";
import { devtools } from "zustand/middleware";
import type { WeightPreset } from "@/stores/types";

export type { WeightPreset };
export type RiskLevel = "low" | "mid" | "high";

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
    // TODO: prefecture is kept here for test compatibility. prefecture-store is
    // the authoritative source of truth for the selected prefecture once
    // multi-prefecture support is added. At that point, remove this field and
    // update filter-store.test.ts assertions accordingly.
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

function computeActivePredicates(s: FilterState): boolean[] {
  return [
    s.area.cities.length > 0,
    s.area.customPolygon !== null,
    s.criteria.tlsMin > DEFAULTS.criteria.tlsMin,
    s.criteria.riskMax !== DEFAULTS.criteria.riskMax,
    s.criteria.priceRange[0] > DEFAULTS.criteria.priceRange[0] ||
      s.criteria.priceRange[1] < DEFAULTS.criteria.priceRange[1],
    s.zoning.zones.length > 0,
    s.zoning.stationMaxDistanceM < DEFAULTS.zoning.stationMaxDistanceM,
    s.preset !== DEFAULTS.preset,
  ];
}

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

      isActive: () => computeActivePredicates(get()).some(Boolean),

      activeCount: () => computeActivePredicates(get()).filter(Boolean).length,

      toQueryParams: () => {
        const s = get();
        const p: Record<string, string> = {};
        if (s.criteria.tlsMin > DEFAULTS.criteria.tlsMin)
          p.tls_min = String(s.criteria.tlsMin);
        if (s.criteria.riskMax !== DEFAULTS.criteria.riskMax)
          p.risk_max = s.criteria.riskMax;
        if (s.zoning.zones.length > 0) p.zones = s.zoning.zones.join(",");
        if (
          s.zoning.stationMaxDistanceM < DEFAULTS.zoning.stationMaxDistanceM
        ) {
          p.station_max = String(s.zoning.stationMaxDistanceM);
        }
        if (s.criteria.priceRange[0] > DEFAULTS.criteria.priceRange[0])
          p.price_min = String(s.criteria.priceRange[0]);
        if (s.criteria.priceRange[1] < DEFAULTS.criteria.priceRange[1])
          p.price_max = String(s.criteria.priceRange[1]);
        if (s.preset !== DEFAULTS.preset) p.preset = s.preset;
        if (s.area.cities.length > 0) p.cities = s.area.cities.join(",");
        return p;
      },
    }),
    { name: "filter-store" },
  ),
);
