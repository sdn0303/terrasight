import { PRICE_STOPS } from "@/lib/palette";

/** Price-to-color interpolation stops for MapLibre paint expressions.
 * Format: [price_threshold, hex_color, ...] — alternating value/color pairs.
 * Used by both the 3D extrusion layer and the year slider legend. */
export const PRICE_COLOR_STOPS = [
  100000,
  PRICE_STOPS.min, // blue — low
  300000,
  "#22c55e", // green — mid
  500000,
  PRICE_STOPS.midHigh, // yellow — high
  1000000,
  PRICE_STOPS.max, // red — very high
  3000000,
  "#a855f7", // purple — premium
] as const;

export const PRICE_HEIGHT_STOPS = [
  0, 0, 100000, 20, 300000, 60, 500000, 100, 1000000, 160, 3000000, 200,
] as const;

/** Legend display labels for each price band. */
export const PRICE_LEGEND = [
  { color: PRICE_STOPS.min, label: "~10万" },
  { color: "#22c55e", label: "~30万" },
  { color: PRICE_STOPS.midHigh, label: "~50万" },
  { color: PRICE_STOPS.max, label: "~100万" },
  { color: "#a855f7", label: "300万~" },
] as const;
