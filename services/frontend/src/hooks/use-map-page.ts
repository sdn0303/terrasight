"use client";

import type { FeatureCollection } from "geojson";
import { parseAsInteger, useQueryState } from "nuqs";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useAreaData } from "@/features/area-data/api/use-area-data";
import { useHealth } from "@/features/health/api/use-health";
import { useLandPricesAllYears } from "@/features/land-prices/api/use-land-prices-all-years";
import { useMapUrlState } from "@/hooks/use-map-url-state";
import type { BBox } from "@/lib/api";
import type { LayerConfig } from "@/lib/layers";
import { LAYERS } from "@/lib/layers";
import { logger } from "@/lib/logger";
import { spatialEngine } from "@/lib/wasm/spatial-engine";
import { useMapStore } from "@/stores/map-store";

const log = logger.child({ module: "use-map-page" });

const EMPTY_FC: FeatureCollection = {
  type: "FeatureCollection",
  features: [],
};

const INTERACTIVE_LAYER_MAP = new Map<string, LayerConfig>();
for (const layer of LAYERS) {
  if (layer.interactiveLayerIds) {
    for (const maplibreId of layer.interactiveLayerIds) {
      INTERACTIVE_LAYER_MAP.set(maplibreId, layer);
    }
  }
}

export function useMapPage() {
  useMapUrlState();

  const [wasmError, setWasmError] = useState(false);
  useEffect(() => {
    spatialEngine.init().catch((err: unknown) => {
      log.error({ err }, "WASM spatial engine failed to initialize");
      setWasmError(true);
    });
    return () => spatialEngine.dispose();
  }, []);

  const visibleLayers = useMapStore((s) => s.visibleLayers);
  const viewState = useMapStore((s) => s.viewState);
  const selectedFeature = useMapStore((s) => s.selectedFeature);

  const [bbox, setBbox] = useState<BBox>(() =>
    useMapStore.getState().getBBox(),
  );
  const handleMoveEnd = useCallback((newBbox: BBox) => {
    setBbox(newBbox);
  }, []);

  const [populationYear, setPopulationYear] = useState(2020);
  const [landPriceYear, setLandPriceYear] = useQueryState(
    "year",
    parseAsInteger.withDefault(2024),
  );

  const layers = useMemo(() => [...visibleLayers], [visibleLayers]);
  const {
    data: areaData,
    isLoading,
    isError: areaDataError,
  } = useAreaData(bbox, layers, viewState.zoom);
  const { data: health } = useHealth();
  // Time machine: fetch all years in a single request so the slider can
  // filter client-side via MapLibre setFilter without refetching per tick.
  const {
    data: landPriceData,
    isFetching: isLandPriceFetching,
    isError: isLandPriceError,
  } = useLandPricesAllYears(bbox, 2020, 2024, viewState.zoom);

  const isZoomTooLow = viewState.zoom < 10;
  const isDemoMode = health ? !health.reinfolib_key_set : true;

  const truncatedLayers = useMemo(() => {
    if (!areaData) return [];
    const result: { layer: string; count: number; limit: number }[] = [];
    for (const key of Object.keys(areaData) as (keyof typeof areaData)[]) {
      const layer = areaData[key];
      if (layer?.truncated === true) {
        result.push({ layer: key, count: layer.count, limit: layer.limit });
      }
    }
    return result;
  }, [areaData]);

  const selectedLayerConfig = useMemo(() => {
    if (!selectedFeature) return null;
    return (
      INTERACTIVE_LAYER_MAP.get(selectedFeature.layerId) ??
      LAYERS.find((l) => selectedFeature.layerId.startsWith(l.id)) ??
      null
    );
  }, [selectedFeature]);

  const staticLayers = useMemo(
    () => LAYERS.filter((l) => l.source === "static"),
    [],
  );
  const apiLayers = useMemo(() => LAYERS.filter((l) => l.source === "api"), []);

  return {
    viewState,
    visibleLayers,
    selectedFeature,
    selectedLayerConfig,
    areaData,
    landPriceData: landPriceData ?? EMPTY_FC,
    isLoading,
    isLandPriceFetching,
    isLandPriceError,
    isZoomTooLow,
    isDemoMode,
    truncatedLayers,
    wasmError,
    areaDataError,
    populationYear,
    setPopulationYear,
    landPriceYear,
    setLandPriceYear,
    handleMoveEnd,
    staticLayers,
    apiLayers,
  };
}
