"use client";

import { useMapStore } from "@/stores/map-store";
import { usePrefectureStore } from "@/stores/prefecture-store";
import { useUIStore } from "@/stores/ui-store";

export type NavigationLevel = "L1" | "L2" | "L3" | "L4";

/** L2 display minimum zoom level */
const L2_MIN_ZOOM = 8;
/** L3 display minimum zoom level */
const L3_MIN_ZOOM = 12;

/**
 * Determines the current navigation level based on map zoom, selected prefecture,
 * and insight context.
 *
 * Level hierarchy:
 * - L1: National (all prefectures, zoom < 8)
 * - L2: Prefecture (selected prefecture, zoom 8-11)
 * - L3: Municipality (zoom >= 12)
 * - L4: Property detail (insight open, always shown)
 */
export function useNavigationLevel(): NavigationLevel {
  const zoom = useMapStore((s) => s.viewState.zoom);
  const leftPanel = useUIStore((s) => s.leftPanel);
  const selectedPref = usePrefectureStore((s) => s.selectedPrefCode);

  // L4 takes priority when the left panel is open
  if (leftPanel !== null) {
    return "L4";
  }

  // L3 when zoom is sufficient for municipality view
  if (zoom >= L3_MIN_ZOOM) {
    return "L3";
  }

  // L2 when prefecture is selected and zoom is sufficient
  if (selectedPref && zoom >= L2_MIN_ZOOM) {
    return "L2";
  }

  // Default to L1 (national view)
  return "L1";
}
