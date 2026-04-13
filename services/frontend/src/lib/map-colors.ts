/**
 * MapLibre paint spec 用の色定数。
 *
 * globals.css の design tokens と同期を維持すること。
 * MapLibre の paint/layout spec は CSS 変数を受け付けないため、
 * ここで JS 定数として定義する。
 *
 * Sync targets in globals.css :root:
 *   --score-good-start  --score-mid-start  --score-bad-start
 */

/** スコア系グラデーション（good=高スコア、bad=低スコア） */
export const SCORE_PALETTE = {
  good: "#10b981", // --score-good-start
  mid: "#f59e0b", // --score-mid-start
  bad: "#ef4444", // --score-bad-start
} as const;

/** Choropleth のデフォルト透明度 */
export const CHOROPLETH_FILL_OPACITY = 0.6;
export const CHOROPLETH_LINE_OPACITY = 0.8;
export const CHOROPLETH_LINE_WIDTH = 1;

/** ラベルのフォントサイズ */
export const LABEL_FONT_SIZE_SM = 10;
export const LABEL_FONT_SIZE_MD = 12;
