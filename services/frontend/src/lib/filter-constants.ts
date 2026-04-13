/**
 * Shared constants and helpers for Finder filter state.
 *
 * Single source of truth for allowed values (wards, zones) and numeric
 * range bounds (TLS score, station distance, price). Consumed by:
 *   - Finder form components (CityMultiSelect, ZoneChips)
 *   - URL round-trip normalizer (use-map-url-state.ts)
 */

export const TOKYO_23_WARDS = [
  "千代田区",
  "中央区",
  "港区",
  "新宿区",
  "文京区",
  "台東区",
  "墨田区",
  "江東区",
  "品川区",
  "目黒区",
  "大田区",
  "世田谷区",
  "渋谷区",
  "中野区",
  "杉並区",
  "豊島区",
  "北区",
  "荒川区",
  "板橋区",
  "練馬区",
  "足立区",
  "葛飾区",
  "江戸川区",
] as const;

/**
 * Cities grouped by prefecture code. Keyed by 2-digit pref code string.
 * Extend when additional prefecture datasets are onboarded.
 */
export const CITIES_BY_PREF: Record<string, readonly string[]> = {
  "13": TOKYO_23_WARDS,
  // 他都道府県は追加時に拡張
};

export const ZONE_OPTIONS = ["商業", "近商", "住居", "工業"] as const;

export type Zone = (typeof ZONE_OPTIONS)[number];
export type TokyoWard = (typeof TOKYO_23_WARDS)[number];

/** TLS score is 0-100 (slider step 5, but any integer in-range is valid via URL). */
export const TLS_MIN_RANGE = { min: 0, max: 100 } as const;

/** Station walking distance in meters. */
export const STATION_MAX_RANGE = { min: 100, max: 2000 } as const;

/** Price is ¥/㎡; upper bound matches the max slider. */
export const PRICE_RANGE = { min: 0, max: 10_000_000 } as const;

/**
 * Clamp an integer into [min, max]. Non-finite input (NaN / ±Infinity)
 * returns `min` so downstream consumers never see an unchecked number.
 */
export function clampInt(value: number, min: number, max: number): number {
  if (!Number.isFinite(value)) return min;
  return Math.max(min, Math.min(max, Math.trunc(value)));
}
