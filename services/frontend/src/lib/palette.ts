export const BRAND = {
  primary: "#6366f1",
  primaryLight: "#818cf8",
  primaryDark: "#4f46e5",
} as const;

export const NEUTRAL = {
  bg: "#ffffff",
  text: "#1e293b",
  textSecondary: "#64748b",
  border: "#e2e8f0",
  darkBg: "#0c0c14",
  darkBgSecondary: "#13131e",
  darkText: "#e4e4e7",
  darkTextSecondary: "#a1a1aa",
  darkBorder: "#27272a",
} as const;

export const SEMANTIC = {
  success: "#10b981",
  warning: "#eab308",
  error: "#ef4444",
  info: "#3b82f6",
} as const;

export const TREND = {
  up: "#10b981",
  down: "#ef4444",
  flat: "#64748b",
} as const;

export const LAYER = {
  flood: "#3b82f6",
  steepSlope: "#f97316",
  landslide: "#a855f7",
  liquefaction: "#eab308",
  station: "#06b6d4",
  railway: "#64748b",
} as const;

export const PRICE_STOPS = {
  min: "#3b82f6",
  low: "#06b6d4",
  mid: "#10b981",
  midHigh: "#eab308",
  high: "#f97316",
  max: "#ef4444",
} as const;

export const OVERLAY = {
  shadow: "rgba(0,0,0,0.08)",
  shadowDark: "rgba(0,0,0,0.4)",
  hover: "rgba(59,130,246,0.06)",
  hoverDark: "rgba(59,130,246,0.12)",
  active: "rgba(99,102,241,0.12)",
  activeDark: "rgba(99,102,241,0.20)",
} as const;

export const BRAND_TINT = {
  light: "rgba(99,102,241,0.06)",
  medium: "rgba(99,102,241,0.12)",
  dark: "rgba(99,102,241,0.20)",
} as const;

export const SCORE = {
  low: "#ef4444",
  mid: "#eab308",
  high: "#10b981",
} as const;

/**
 * Domain stop values (¥/m²) for the land-price color interpolation.
 * Keys match the PRICE_STOPS color keys exactly.
 */
export const PRICE_THRESHOLDS = {
  min: 50_000,
  low: 200_000,
  mid: 500_000,
  midHigh: 1_000_000,
  high: 2_000_000,
  max: 5_000_000,
} as const satisfies Record<keyof typeof PRICE_STOPS, number>;

/**
 * Domain stop values (transaction count) for the transaction-polygon color
 * interpolation. Paired with PRICE_STOPS which reuses the same color ramp.
 */
export const TX_COUNT_THRESHOLDS = {
  min: 10,
  low: 50,
  mid: 100,
  midHigh: 300,
  high: 500,
  max: 1_000,
} as const satisfies Record<keyof typeof PRICE_STOPS, number>;
