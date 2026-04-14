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

export const NARROW_BREAKPOINT = 1280;

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

/**
 * Edge-inset padding in pixels, directly usable as the `padding` option
 * of MapLibre `fitBounds`. Each field is the number of pixels from the
 * respective viewport edge that is hidden by an overlay surface.
 */
export interface MapOverlayPadding {
  top: number;
  right: number;
  bottom: number;
  left: number;
}

/**
 * Compute the padding (in pixels) that MapLibre `fitBounds` needs so that
 * the fitted bounds land inside the visible map area — i.e. not behind the
 * rail, an open left panel, the insight drawer, or an open bottom sheet.
 *
 * Returns edge insets: each field is the distance from that viewport edge
 * to the nearest overlay. These values match MapLibre's
 * `fitBounds({ padding })` contract and can be passed through directly.
 *
 * Note: the `viewport` height is only needed to derive the bottom-sheet
 * pixel height from its percentage; the width is not currently used but
 * is kept in the signature for forward compatibility with right-side
 * percentage-sized overlays.
 */
export function mapOverlayPaddingPx(
  ui: VisibleBoundsInput,
  viewport: { w: number; h: number },
): MapOverlayPadding {
  const left = ui.leftPanel !== null ? MAIN_LEFT_WITH_PANEL : MAIN_LEFT_BASE;
  const right = ui.insight !== null ? MAIN_RIGHT_WITH_DRAWER : MAIN_RIGHT_BASE;
  const bottom =
    ui.bottomSheet !== null
      ? (ui.bottomSheetHeightPct / 100) * viewport.h + PAGE_INSET
      : PAGE_INSET;
  return {
    top: PAGE_INSET,
    right,
    bottom,
    left,
  };
}
