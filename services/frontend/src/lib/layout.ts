/**
 * Layout constants for the single-page map-canvas UI shell.
 *
 * All Layer 1/2 surfaces (rail, panels, drawer, bottom sheet) are positioned
 * as floating cards on top of the always-visible MapLibre canvas. These
 * constants define the spatial grid that relates them.
 */

export const PAGE_INSET = 20;
export const RAIL_WIDTH = 72;
export const GAP = 16;

export const LEFT_PANEL_WIDTH = 360;
export const DRAWER_WIDTH = 340;

export const BOTTOM_SHEET_DEFAULT_PCT = 40;
export const BOTTOM_SHEET_MIN_PCT = 20;
export const BOTTOM_SHEET_MAX_PCT = 80;

export const MAIN_LEFT_BASE = PAGE_INSET + RAIL_WIDTH + GAP; // 108
export const MAIN_LEFT_WITH_PANEL = MAIN_LEFT_BASE + LEFT_PANEL_WIDTH + GAP; // 484
export const MAIN_RIGHT_BASE = PAGE_INSET; // 20
export const MAIN_RIGHT_WITH_DRAWER = PAGE_INSET + DRAWER_WIDTH + GAP; // 376

export const CARD_RADIUS = {
  rail: 22,
  mainPanel: 24,
  drawer: 24,
  bottomSheet: 24,
  smallChip: 14,
  button: 12,
  pill: 999,
} as const;

export const CARD_SHADOW = {
  strong: "0 40px 100px rgba(0,0,0,0.45), 0 0 0 1px rgba(255,255,255,0.5)",
  medium: "0 24px 60px rgba(0,0,0,0.35), 0 0 0 1px rgba(255,255,255,0.4)",
  subtle: "0 10px 24px rgba(0,0,0,0.18)",
} as const;

/**
 * Minimal shape of the ui-store state needed to compute the visible map
 * bounds. This utility only cares whether each overlay is active, not
 * what variant it is, so the fields accept any non-null marker.
 *
 * In practice, pass the current `useUIStore` snapshot directly — the
 * real store types (string literal unions, insight discriminated union)
 * are structurally compatible with these looser types.
 */
export interface VisibleBoundsInput {
  leftPanel: string | null;
  insight: object | null;
  bottomSheet: string | null;
  bottomSheetHeightPct: number;
}

export interface VisibleBoundsPx {
  left: number;
  right: number;
  top: number;
  bottom: number;
}

/**
 * Compute the map area that is actually visible (not hidden behind overlays)
 * so MapLibre `fitBounds` can pad correctly.
 */
export function visibleMapBoundsPx(
  ui: VisibleBoundsInput,
  viewport: { w: number; h: number },
): VisibleBoundsPx {
  const leftOffset =
    ui.leftPanel !== null ? MAIN_LEFT_WITH_PANEL : MAIN_LEFT_BASE;
  const rightOffset =
    ui.insight !== null ? MAIN_RIGHT_WITH_DRAWER : MAIN_RIGHT_BASE;
  const bottomOffset =
    ui.bottomSheet !== null
      ? (ui.bottomSheetHeightPct / 100) * viewport.h + PAGE_INSET
      : PAGE_INSET;
  return {
    left: leftOffset,
    right: viewport.w - rightOffset,
    top: PAGE_INSET,
    bottom: viewport.h - bottomOffset,
  };
}
