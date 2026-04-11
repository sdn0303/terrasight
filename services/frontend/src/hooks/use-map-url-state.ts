"use client";

import {
  parseAsArrayOf,
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
import { useFilterStore } from "@/stores/filter-store";
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
  // Finder / filter params (Phase 3)
  tlsMin: parseAsInteger,
  riskMax: parseAsStringLiteral(["low", "mid", "high"] as const),
  zones: parseAsArrayOf(parseAsString),
  stationMax: parseAsInteger,
  priceMin: parseAsInteger,
  priceMax: parseAsInteger,
  preset: parseAsStringLiteral([
    "balance",
    "investment",
    "residential",
    "disaster",
  ] as const),
  cities: parseAsArrayOf(parseAsString),
  panel: parseAsStringLiteral(["finder", "layers", "themes"] as const),
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
  const comparePoints = useUIStore((s) => s.comparePoints);
  const insight = useUIStore((s) => s.insight);
  const activeTab = useUIStore((s) => s.activeTab);

  // Subscribe to filter-store slices so the sync effect re-runs on changes.
  // We call getState().toQueryParams() inside the effect body to get a fresh
  // snapshot; subscribing to these slices ensures re-runs because Zustand
  // produces a new object reference on each setArea/setCriteria/setZoning/setPreset.
  const filterArea = useFilterStore((s) => s.area);
  const filterCriteria = useFilterStore((s) => s.criteria);
  const filterZoning = useFilterStore((s) => s.zoning);
  const filterPreset = useFilterStore((s) => s.preset);
  const leftPanel = useUIStore((s) => s.leftPanel);

  // On mount: restore map state from URL
  // biome-ignore lint/correctness/useExhaustiveDependencies: mount-once pattern intentionally reads URL params at init time
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

    // Restore filter state from URL (Phase 3)
    const filterStore = useFilterStore.getState();
    if (
      params.tlsMin !== null ||
      params.riskMax !== null ||
      params.priceMin !== null ||
      params.priceMax !== null
    ) {
      filterStore.setCriteria({
        tlsMin: params.tlsMin ?? 0,
        riskMax: params.riskMax ?? "high",
        priceRange: [params.priceMin ?? 0, params.priceMax ?? 10_000_000],
      });
    }
    if (params.zones !== null && params.zones.length > 0) {
      filterStore.setZoning({ zones: params.zones });
    }
    if (params.stationMax !== null) {
      filterStore.setZoning({ stationMaxDistanceM: params.stationMax });
    }
    if (params.preset !== null) {
      filterStore.setPreset(params.preset);
    }
    if (params.cities !== null && params.cities.length > 0) {
      filterStore.setArea({ cities: params.cities });
    }
    if (params.panel !== null) {
      useUIStore.getState().setLeftPanel(params.panel);
    }
  }, []);

  // Sync store → URL on state change
  // biome-ignore lint/correctness/useExhaustiveDependencies: filterArea/Criteria/Zoning/Preset trigger re-runs; toQueryParams() is called via getState() inside the body
  useEffect(() => {
    if (!initialized.current) return;
    // Compute fresh filter params from getState(); the selectors above
    // (filterArea, filterCriteria, filterZoning, filterPreset) ensure this
    // effect re-runs whenever filter state changes by reference.
    const filterParams = useFilterStore.getState().toQueryParams();
    setParams({
      lat: Math.round(viewState.latitude * 10000) / 10000,
      lng: Math.round(viewState.longitude * 10000) / 10000,
      z: Math.round(viewState.zoom * 10) / 10,
      pitch: Math.round(viewState.pitch),
      bearing: Math.round(viewState.bearing),
      layers: [...visibleLayers].sort().join(","),
      theme: [...activeThemes].sort().join(","),
      mode,
      // Phase 2a: alat/alng reflect the Insight drawer selection only, so
      // closing the drawer (setInsight(null)) clears them from the URL.
      // The legacy mapStore.analysisPoint is still updated by
      // useMapInteraction for back-compat with non-rendered legacy panels
      // but is intentionally not read here; on mount we restore both from
      // alat/alng in lockstep.
      alat: insight?.lat ?? null,
      alng: insight?.lng ?? null,
      cp:
        comparePoints.length > 0
          ? comparePoints.map((p) => `${p.lat},${p.lng},${p.address}`).join("|")
          : "",
      tab: activeTab,
      // Phase 3: filter params
      tlsMin:
        filterParams.tls_min !== undefined
          ? Number(filterParams.tls_min)
          : null,
      riskMax:
        (filterParams.risk_max as "low" | "mid" | "high" | undefined) ?? null,
      zones:
        filterParams.zones !== undefined ? filterParams.zones.split(",") : null,
      stationMax:
        filterParams.station_max !== undefined
          ? Number(filterParams.station_max)
          : null,
      priceMin:
        filterParams.price_min !== undefined
          ? Number(filterParams.price_min)
          : null,
      priceMax:
        filterParams.price_max !== undefined
          ? Number(filterParams.price_max)
          : null,
      preset:
        (filterParams.preset as
          | "balance"
          | "investment"
          | "residential"
          | "disaster"
          | undefined) ?? null,
      cities:
        filterParams.cities !== undefined
          ? filterParams.cities.split(",")
          : null,
      panel: leftPanel,
    });
  }, [
    viewState,
    visibleLayers,
    activeThemes,
    mode,
    comparePoints,
    insight,
    activeTab,
    filterArea,
    filterCriteria,
    filterZoning,
    filterPreset,
    leftPanel,
    setParams,
  ]);
}
