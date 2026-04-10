/**
 * Theme token recipes for inline-style consumption by React components.
 *
 * Most color and surface tokens live in globals.css as CSS variables. This
 * file provides TypeScript-level recipes (gradients, glow shadows, score-to-
 * color mappings) that need to be passed as strings to inline styles.
 */

export const GRADIENT = {
  brand: "linear-gradient(135deg, #6366f1 0%, #8b5cf6 55%, #06b6d4 100%)",
  primary: "linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%)",
  success: "linear-gradient(135deg, #10b981 0%, #059669 100%)",
  warn: "linear-gradient(135deg, #f59e0b 0%, #d97706 100%)",
  danger: "linear-gradient(135deg, #ef4444 0%, #dc2626 100%)",
  hot: "linear-gradient(135deg, #f43f5e 0%, #e11d48 100%)",
  warm: "linear-gradient(135deg, #fb923c 0%, #f97316 100%)",
  tableHeader: "linear-gradient(90deg, #312e81 0%, #4c1d95 50%, #5b21b6 100%)",
  brandTint:
    "linear-gradient(135deg, rgba(99,102,241,0.16) 0%, rgba(168,85,247,0.16) 100%)",
  heroCard: "linear-gradient(135deg, #eef2ff 0%, #f5f3ff 50%, #eff6ff 100%)",
  neutral: "linear-gradient(135deg, #fef3c7 0%, #fde68a 100%)",
  cold: "linear-gradient(135deg, #e2e8f0 0%, #cbd5e1 100%)",
} as const;

export const GLOW_SHADOW = {
  primary: "0 12px 28px rgba(99,102,241,0.45)",
  primarySubtle: "0 8px 20px rgba(99,102,241,0.35)",
  success: "0 6px 14px rgba(16,185,129,0.35)",
  warn: "0 6px 14px rgba(245,158,11,0.35)",
  danger: "0 6px 14px rgba(239,68,68,0.35)",
  hot: "0 6px 16px rgba(244,63,94,0.4)",
} as const;

const SCORE_THRESHOLD = {
  WARN: 40,
  BRAND: 60,
  SUCCESS: 80,
} as const;

/**
 * Returns the gradient background for a 0-100 score based on these ranges:
 *   0-39   → danger (red)
 *   40-59  → warn (amber)
 *   60-79  → brand (indigo)
 *   80-100 → success (emerald)
 *
 * Values outside 0-100 are clamped.
 */
export function scoreGradient(value: number): string {
  const clamped = Math.max(0, Math.min(100, value));
  if (clamped < SCORE_THRESHOLD.WARN) return GRADIENT.danger;
  if (clamped < SCORE_THRESHOLD.BRAND) return GRADIENT.warn;
  if (clamped < SCORE_THRESHOLD.SUCCESS) return GRADIENT.brand;
  return GRADIENT.success;
}

/**
 * Returns the gradient background for a signal tier pill.
 */
export function signalGradient(
  signal: "hot" | "warm" | "neutral" | "cold",
): string {
  switch (signal) {
    case "hot":
      return GRADIENT.hot;
    case "warm":
      return GRADIENT.warm;
    case "neutral":
      return GRADIENT.neutral;
    case "cold":
      return GRADIENT.cold;
  }
}
