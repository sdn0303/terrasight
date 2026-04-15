"use client";

import { parseAsInteger, useQueryState } from "nuqs";
import { memo, useMemo } from "react";
import { LayerRenderer } from "@/components/map/layer-renderer";
import { useAreaData } from "@/features/area-data/api/use-area-data";
import { useLandPricesAllYears } from "@/features/land-prices/api/use-land-prices-all-years";
import type { BBox } from "@/lib/api";
import { LAND_PRICE_FROM_YEAR, LAND_PRICE_TO_YEAR } from "@/lib/constants";
import { EMPTY_FC } from "@/lib/geo-constants";
import { LAYERS } from "@/lib/layers";
import { useMapStore } from "@/stores/map-store";

const staticLayers = LAYERS.filter((l) => l.source === "static");
const apiLayers = LAYERS.filter((l) => l.source === "api");

/**
 * Data-fetching wrapper for map layers.
 *
 * Isolated from MapView's per-frame viewState re-renders by:
 * - Receiving bbox only on moveEnd (debounced in MapView)
 * - Using integer zoom (changes ~once per zoom step, not every frame)
 * - Being wrapped in React.memo so parent re-renders don't cascade
 */
export const MapLayers = memo(function MapLayers({
  bbox,
}: {
  bbox: BBox | null;
}) {
  const visibleLayers = useMapStore((s) => s.visibleLayers);
  // Integer zoom changes much less frequently than fractional zoom,
  // preventing per-frame re-renders during pinch-to-zoom.
  const zoom = useMapStore((s) => Math.floor(s.viewState.zoom));

  const [populationYear] = useQueryState(
    "popYear",
    parseAsInteger.withDefault(2020),
  );
  const [landPriceYear] = useQueryState(
    "year",
    parseAsInteger.withDefault(2026),
  );

  // Only send API-sourced layer IDs to area-data endpoint.
  // Static and timeseries layers are fetched via separate hooks.
  const apiLayerIds = useMemo(() => {
    const apiIds = new Set(apiLayers.map((l) => l.id));
    return [...visibleLayers].filter((id) => apiIds.has(id));
  }, [visibleLayers]);

  const { data: areaData } = useAreaData(bbox, apiLayerIds, zoom);
  const { data: landPriceData, isFetching: isLandPriceFetching } =
    useLandPricesAllYears(bbox, LAND_PRICE_FROM_YEAR, LAND_PRICE_TO_YEAR, zoom);

  return (
    <LayerRenderer
      visibleLayers={visibleLayers}
      staticLayers={staticLayers}
      apiLayers={apiLayers}
      areaData={areaData ?? null}
      landPriceData={landPriceData ?? EMPTY_FC}
      isLandPriceFetching={isLandPriceFetching}
      populationYear={populationYear}
      landPriceYear={landPriceYear}
    />
  );
});
