import { LAYERS } from "./layers";

export type ThemeId = "safety" | "livability" | "price" | "future";

export interface ThemeConfig {
  id: ThemeId;
  /** i18n key for display name */
  labelKey: string;
  /** TLS axis used for choropleth coloring */
  colorAxis: "disaster" | "terrain" | "livability" | "future" | "price";
  /** Color ramp: [low, mid, high] for choropleth */
  colorRamp: [string, string, string];
}

export const THEMES: ThemeConfig[] = [
  { id: "safety", labelKey: "theme.safety", colorAxis: "disaster", colorRamp: ["#ef4444", "#eab308", "#10b981"] },
  { id: "livability", labelKey: "theme.livability", colorAxis: "livability", colorRamp: ["#f97316", "#eab308", "#10b981"] },
  { id: "price", labelKey: "theme.price", colorAxis: "price", colorRamp: ["#3b82f6", "#eab308", "#ef4444"] },
  { id: "future", labelKey: "theme.future", colorAxis: "future", colorRamp: ["#ef4444", "#eab308", "#3b82f6"] },
];

/** Get layer IDs that belong to a given theme */
export function getLayerIdsByTheme(themeId: ThemeId): string[] {
  return LAYERS.filter((l) => l.theme?.includes(themeId)).map((l) => l.id);
}

/** Get all layer IDs for multiple active themes */
export function getLayerIdsForThemes(themeIds: Set<ThemeId>): Set<string> {
  const ids = new Set<string>();
  for (const themeId of themeIds) {
    for (const layerId of getLayerIdsByTheme(themeId)) {
      ids.add(layerId);
    }
  }
  // Always include admin_boundary
  ids.add("admin_boundary");
  return ids;
}
