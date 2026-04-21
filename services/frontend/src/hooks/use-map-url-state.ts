"use client";

import { useEffect, useRef } from "react";
import { isValidTabId } from "@/features/tabs/tab-configs";
import { useMapStore } from "@/stores/map-store";
import { useUIStore } from "@/stores/ui-store";

export function isValidCoordinate(
  lat: number | null,
  lng: number | null,
): boolean {
  return (
    lat != null &&
    lng != null &&
    Number.isFinite(lat) &&
    Number.isFinite(lng) &&
    Math.abs(lat) <= 90 &&
    Math.abs(lng) <= 180
  );
}

/** Sync map viewState and activeTab with URL search params. */
export function useMapUrlState() {
  const initialized = useRef(false);
  const viewState = useMapStore((s) => s.viewState);
  const setViewState = useMapStore((s) => s.setViewState);
  const activeTab = useUIStore((s) => s.activeTab);
  const setActiveTab = useUIStore((s) => s.setActiveTab);

  // Restore from URL on mount
  // biome-ignore lint/correctness/useExhaustiveDependencies: mount-once pattern intentionally reads URL params at init time
  useEffect(() => {
    if (initialized.current) return;
    initialized.current = true;

    const params = new URLSearchParams(window.location.search);
    const lat = params.get("lat");
    const lng = params.get("lng");
    const z = params.get("z");
    const pitch = params.get("pitch");
    const bearing = params.get("bearing");
    const tab = params.get("tab");

    if (lat !== null && lng !== null && z !== null) {
      const parsedLat = Number(lat);
      const parsedLng = Number(lng);
      const parsedZ = Number(z);

      if (isValidCoordinate(parsedLat, parsedLng) && Number.isFinite(parsedZ)) {
        const current = useMapStore.getState().viewState;
        setViewState({
          latitude: parsedLat,
          longitude: parsedLng,
          zoom: parsedZ,
          pitch:
            pitch !== null && Number.isFinite(Number(pitch))
              ? Number(pitch)
              : current.pitch,
          bearing:
            bearing !== null && Number.isFinite(Number(bearing))
              ? Number(bearing)
              : current.bearing,
        });
      }
    }

    if (tab !== null && isValidTabId(tab)) {
      setActiveTab(tab);
    }
  }, []);

  // Write to URL on state change
  useEffect(() => {
    if (!initialized.current) return;
    const params = new URLSearchParams();
    params.set("lat", viewState.latitude.toFixed(4));
    params.set("lng", viewState.longitude.toFixed(4));
    params.set("z", viewState.zoom.toFixed(1));
    if (viewState.pitch > 0) {
      params.set("pitch", viewState.pitch.toFixed(0));
    }
    if (viewState.bearing !== 0) {
      params.set("bearing", viewState.bearing.toFixed(0));
    }
    params.set("tab", activeTab);
    window.history.replaceState(null, "", `?${params.toString()}`);
  }, [
    viewState.latitude,
    viewState.longitude,
    viewState.zoom,
    viewState.pitch,
    viewState.bearing,
    activeTab,
  ]);
}
