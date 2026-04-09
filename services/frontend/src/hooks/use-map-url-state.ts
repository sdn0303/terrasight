"use client";

import {
  parseAsFloat,
  parseAsInteger,
  parseAsString,
  useQueryStates,
} from "nuqs";
import { useEffect, useRef } from "react";
import { MAP_CONFIG } from "@/lib/constants";
import type { ThemeId } from "@/lib/themes";
import { THEMES } from "@/lib/themes";
import { useMapStore } from "@/stores/map-store";
import { useUIStore } from "@/stores/ui-store";

const mapParams = {
  lat: parseAsFloat.withDefault(MAP_CONFIG.center[1]),
  lng: parseAsFloat.withDefault(MAP_CONFIG.center[0]),
  z: parseAsFloat.withDefault(MAP_CONFIG.zoom),
  pitch: parseAsFloat.withDefault(MAP_CONFIG.pitch),
  bearing: parseAsFloat.withDefault(MAP_CONFIG.bearing),
  layers: parseAsString.withDefault("land_price_ts,zoning"),
  theme: parseAsString.withDefault("safety"),
  year: parseAsInteger.withDefault(2024),
  // Analysis context params
  mode: parseAsString.withDefault("explore"),
  alat: parseAsFloat,
  alng: parseAsFloat,
  cp: parseAsString.withDefault(""),
};

export function useMapUrlState() {
  const [params, setParams] = useQueryStates(mapParams, {
    history: "replace",
    shallow: true,
  });
  const initialized = useRef(false);
  const { viewState, setViewState, visibleLayers, toggleLayer } =
    useMapStore();
  const activeThemes = useUIStore((s) => s.activeThemes);
  const mode = useUIStore((s) => s.mode);
  const analysisPoint = useMapStore((s) => s.analysisPoint);
  const comparePoints = useUIStore((s) => s.comparePoints);

  // On mount: restore map state from URL
  useEffect(() => {
    if (initialized.current) return;
    initialized.current = true;

    setViewState({
      latitude: params.lat,
      longitude: params.lng,
      zoom: params.z,
      pitch: params.pitch,
      bearing: params.bearing,
    });

    // Sync layers from URL
    const urlLayers = new Set(params.layers.split(",").filter(Boolean));
    const currentLayers = useMapStore.getState().visibleLayers;
    for (const id of currentLayers) {
      if (!urlLayers.has(id)) toggleLayer(id);
    }
    for (const id of urlLayers) {
      if (!currentLayers.has(id)) toggleLayer(id);
    }

    // Restore theme from URL
    const validThemeIds = new Set<string>(THEMES.map((t) => t.id));
    const themeIds = params.theme
      .split(",")
      .filter((id): id is ThemeId => validThemeIds.has(id as ThemeId));

    const currentThemes = useUIStore.getState().activeThemes;
    if (currentThemes.size > 0) {
      useUIStore.getState().clearThemes();
    }
    for (const themeId of themeIds) {
      useUIStore.getState().toggleTheme(themeId);
    }

    // Restore mode from URL
    if (params.mode === "compare") {
      useUIStore.getState().setMode("compare");
    }

    // Restore analysis point from URL (validate coordinates)
    if (
      params.alat != null &&
      params.alng != null &&
      Number.isFinite(params.alat) &&
      Number.isFinite(params.alng) &&
      Math.abs(params.alat) <= 90 &&
      Math.abs(params.alng) <= 180
    ) {
      useMapStore.getState().setAnalysisPoint({
        lat: params.alat,
        lng: params.alng,
      });
    }

    // Restore compare points from URL
    if (params.cp) {
      const points = params.cp
        .split("|")
        .filter(Boolean)
        .map((s) => {
          const [latStr, lngStr, ...nameParts] = s.split(",");
          const lat = Number(latStr);
          const lng = Number(lngStr);
          return { lat, lng, address: nameParts.join(",") || "Unknown" };
        })
        .filter(
          (pt) =>
            Number.isFinite(pt.lat) &&
            Number.isFinite(pt.lng) &&
            Math.abs(pt.lat) <= 90 &&
            Math.abs(pt.lng) <= 180,
        );
      for (const pt of points.slice(0, 3)) {
        useUIStore.getState().addComparePoint(pt);
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Sync store → URL on state change
  useEffect(() => {
    if (!initialized.current) return;
    setParams({
      lat: Math.round(viewState.latitude * 10000) / 10000,
      lng: Math.round(viewState.longitude * 10000) / 10000,
      z: Math.round(viewState.zoom * 10) / 10,
      pitch: Math.round(viewState.pitch),
      bearing: Math.round(viewState.bearing),
      layers: [...visibleLayers].sort().join(","),
      theme: [...activeThemes].sort().join(","),
      mode,
      alat: analysisPoint?.lat ?? null,
      alng: analysisPoint?.lng ?? null,
      cp:
        comparePoints.length > 0
          ? comparePoints
              .map((p) => `${p.lat},${p.lng},${p.address}`)
              .join("|")
          : "",
    });
  }, [
    viewState,
    visibleLayers,
    activeThemes,
    mode,
    analysisPoint,
    comparePoints,
    setParams,
  ]);
}
