import { useMemo } from "react";
import { useUIStore } from "@/stores/ui-store";
import type { ThemeId } from "@/lib/theme-definitions";

/**
 * Maps each theme to its Mapbox layer IDs.
 * WASM layers use FlatGeobuf sources, API layers use GeoJSON sources.
 * See docs/designs/map-visualization-spec.md for layer style details.
 */
const THEME_LAYERS: Record<ThemeId, string[]> = {
  "land-price": ["land_price_polygon"],
  hazard: [
    "flood",
    "flood_history",
    "steep_slope",
    "landslide",
    "liquefaction",
  ],
  transactions: ["transaction_polygon"],
  station: ["station", "railway"],
  score: [],
};

export function useThemeLayers() {
  const activeTheme = useUIStore((s) => s.activeTheme);

  const visibleLayerIds = useMemo(() => {
    if (!activeTheme) return new Set<string>();
    return new Set(THEME_LAYERS[activeTheme] ?? []);
  }, [activeTheme]);

  return { activeTheme, visibleLayerIds };
}
