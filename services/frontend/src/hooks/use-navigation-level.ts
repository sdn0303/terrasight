"use client";

import { useMapStore } from "@/stores/map-store";
import { usePrefectureStore } from "@/stores/prefecture-store";
import { useUIStore } from "@/stores/ui-store";

export type NavigationLevel = "L1" | "L2" | "L3" | "L4";

const L2_MIN_ZOOM = 8;
const L3_MIN_ZOOM = 12;

/**
 * Determines the current navigation level based on map zoom, selected prefecture,
 * and detail panel state.
 */
export function useNavigationLevel(): NavigationLevel {
  const zoom = useMapStore((s) => s.viewState.zoom);
  const selectedArea = useUIStore((s) => s.selectedArea);
  const selectedPref = usePrefectureStore((s) => s.selectedPrefCode);

  if (selectedArea !== null) {
    return "L4";
  }

  if (zoom >= L3_MIN_ZOOM) {
    return "L3";
  }

  if (selectedPref && zoom >= L2_MIN_ZOOM) {
    return "L2";
  }

  return "L1";
}
