"use client";

import {
  parseAsFloat,
  parseAsInteger,
  parseAsString,
  parseAsStringLiteral,
  useQueryStates,
} from "nuqs";
import { useEffect, useRef } from "react";
import { MAP_CONFIG } from "@/lib/constants";
import type { ThemeId } from "@/lib/themes";
import { THEMES } from "@/lib/themes";
import { useMapStore } from "@/stores/map-store";
import { type DrawerTab, useUIStore } from "@/stores/ui-store";

const DRAWER_TABS = [
  "intel",
  "trend",
  "risk",
  "infra",
] as const satisfies readonly DrawerTab[];

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
  // Insight drawer tab (Phase 2a)
  tab: parseAsStringLiteral(DRAWER_TABS).withDefault("intel"),
};

export interface ParsedComparePoint {
  lat: number;
  lng: number;
  address: string;
}

/** True when (lat, lng) is a finite number within Earth bounds. */
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

/**
 * Parse the `cp` URL param into validated compare points.
 * Format: `lat,lng,address|lat,lng,address|...` (max 3 entries).
 * Entries with invalid coordinates are filtered out.
 */
export function parseComparePointsParam(cp: string): ParsedComparePoint[] {
  if (!cp) return [];
  return cp
    .split("|")
    .filter(Boolean)
    .map((s) => {
      const [latStr, lngStr, ...nameParts] = s.split(",");
      const lat = Number(latStr);
      const lng = Number(lngStr);
      return { lat, lng, address: nameParts.join(",") || "Unknown" };
    })
    .filter((pt) => isValidCoordinate(pt.lat, pt.lng))
    .slice(0, 3);
}

export function useMapUrlState() {
  const [params, setParams] = useQueryStates(mapParams, {
    history: "replace",
    shallow: true,
  });
  const initialized = useRef(false);
  const { viewState, setViewState, visibleLayers, toggleLayer } = useMapStore();
  const activeThemes = useUIStore((s) => s.activeThemes);
  const mode = useUIStore((s) => s.mode);
  const analysisPoint = useMapStore((s) => s.analysisPoint);
  const comparePoints = useUIStore((s) => s.comparePoints);
  const insight = useUIStore((s) => s.insight);
  const activeTab = useUIStore((s) => s.activeTab);

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
    if (isValidCoordinate(params.alat, params.alng)) {
      // isValidCoordinate guarantees both are non-null finite numbers.
      const alat = params.alat as number;
      const alng = params.alng as number;
      useMapStore.getState().setAnalysisPoint({
        lat: alat,
        lng: alng,
      });
      // Also open the Insight drawer at the same point (Phase 2a)
      useUIStore.getState().setInsight({
        kind: "point",
        lat: alat,
        lng: alng,
      });
    }

    // Restore active tab from URL (Phase 2a)
    useUIStore.getState().setActiveTab(params.tab);

    // Restore compare points from URL (capped at 3, invalid entries dropped)
    const comparePoints = parseComparePointsParam(params.cp);
    for (const pt of comparePoints) {
      useUIStore.getState().addComparePoint(pt);
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
      // Phase 2a: prefer insight over legacy analysisPoint so closing the
      // drawer clears alat/alng from the URL.
      alat: insight?.lat ?? analysisPoint?.lat ?? null,
      alng: insight?.lng ?? analysisPoint?.lng ?? null,
      cp:
        comparePoints.length > 0
          ? comparePoints.map((p) => `${p.lat},${p.lng},${p.address}`).join("|")
          : "",
      tab: activeTab,
    });
  }, [
    viewState,
    visibleLayers,
    activeThemes,
    mode,
    analysisPoint,
    comparePoints,
    insight,
    activeTab,
    setParams,
  ]);
}
