/** Price-to-color interpolation stops for MapLibre paint expressions.
 * Format: [price_threshold, hex_color, ...] — alternating value/color pairs.
 * Used by both the 3D extrusion layer and the year slider legend. */
export const PRICE_COLOR_STOPS = [
  100000,
  "#3b82f6", // blue — low
  300000,
  "#22c55e", // green — mid
  500000,
  "#eab308", // yellow — high
  1000000,
  "#ef4444", // red — very high
  3000000,
  "#a855f7", // purple — premium
] as const;

export const PRICE_HEIGHT_STOPS = [
  0, 0, 100000, 20, 300000, 60, 500000, 100, 1000000, 160, 3000000, 200,
] as const;

/** Legend display labels for each price band. */
export const PRICE_LEGEND = [
  { color: "#3b82f6", label: "~10万" },
  { color: "#22c55e", label: "~30万" },
  { color: "#eab308", label: "~50万" },
  { color: "#ef4444", label: "~100万" },
  { color: "#a855f7", label: "300万~" },
] as const;
