"use client";

import type { FeatureCollection } from "geojson";
import { parseAsInteger, useQueryState } from "nuqs";
import { memo, useMemo, useState } from "react";
import { LayerRenderer } from "@/components/map/layer-renderer";
import { useAreaData } from "@/features/area-data/api/use-area-data";
import { useLandPricesAllYears } from "@/features/land-prices/api/use-land-prices-all-years";
import type { BBox } from "@/lib/api";
import { LAYERS } from "@/lib/layers";
import { useMapStore } from "@/stores/map-store";

const EMPTY_FC: FeatureCollection = {
  type: "FeatureCollection",
  features: [],
};

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

  const [populationYear] = useState(2020);
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
    useLandPricesAllYears(bbox, 2020, 2026, zoom);

  return (
    <LayerRenderer
      visibleLayers={visibleLayers}
      staticLayers={staticLayers}
      apiLayers={apiLayers}
      areaData={areaData as Record<string, unknown> | null}
      landPriceData={landPriceData ?? EMPTY_FC}
      isLandPriceFetching={isLandPriceFetching}
      populationYear={populationYear}
      landPriceYear={landPriceYear}
    />
  );
});
