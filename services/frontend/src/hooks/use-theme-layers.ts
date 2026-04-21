import { useMemo } from "react";
import type { TabId } from "@/features/tabs/tab-configs";
import { useUIStore } from "@/stores/ui-store";

/**
 * Maps each tab to its Mapbox layer IDs.
 * IDs must match layer components registered in LayerRenderer
 * (STATIC_LAYER_COMPONENTS / API_LAYER_COMPONENTS) or new feature layers.
 * See docs/DESIGN.md Sec 6.4 for layer definitions per tab.
 */
const TAB_LAYERS: Record<TabId, string[]> = {
  overview: ["station", "railway"],
  "land-price": ["land_price_polygon", "land_price_polygon_label", "landprice"],
  transactions: ["transaction_polygon", "transaction_polygon_label"],
  population: ["population_mesh", "did"],
  vacancy: [],
  stations: ["station", "railway"],
  yield: [],
  hazard: [
    "flood",
    "flood_history",
    "steep_slope",
    "landslide",
    "liquefaction",
    "seismic",
    "fault",
    "volcano",
  ],
  ground: ["landform", "geology", "soil"],
  zoning: ["zoning"],
};

export function useThemeLayers() {
  const activeTab = useUIStore((s) => s.activeTab);

  const visibleLayerIds = useMemo(() => {
    return new Set(TAB_LAYERS[activeTab] ?? []);
  }, [activeTab]);

  return { activeTab, visibleLayerIds };
}
