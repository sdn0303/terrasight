export const MAP_CONFIG = {
  center: [139.767, 35.681] as [number, number],
  zoom: 12,
  pitch: 45,
  bearing: 0,
  style: "https://basemaps.cartocdn.com/gl/dark-matter-gl-style/style.json",
} as const;

export const DEBOUNCE_MS = 300;
export const BBOX_MAX_DEGREES = 0.5;

/** Inclusive lower bound for land-prices/all-years queries. */
export const LAND_PRICE_FROM_YEAR = 2020;
/** Inclusive upper bound for land-prices/all-years queries. */
export const LAND_PRICE_TO_YEAR = 2026;
