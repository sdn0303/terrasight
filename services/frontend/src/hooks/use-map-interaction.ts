"use client";

import { useCallback } from "react";
import type { MapLayerMouseEvent } from "react-map-gl/maplibre";
import { useMapStore } from "@/stores/map-store";
import { useUIStore } from "@/stores/ui-store";

export function useMapInteraction() {
  const selectFeature = useMapStore((s) => s.selectFeature);
  const setAnalysisPoint = useMapStore((s) => s.setAnalysisPoint);
  const mode = useUIStore((s) => s.mode);
  const setComparePoint = useUIStore((s) => s.setComparePoint);

  const handleFeatureClick = useCallback(
    (e: MapLayerMouseEvent) => {
      const feature = e.features?.[0];

      if (mode === "compare") {
        const address =
          feature?.properties != null &&
          typeof feature.properties === "object" &&
          "address" in feature.properties &&
          typeof feature.properties.address === "string"
            ? feature.properties.address
            : `${e.lngLat.lat.toFixed(4)}, ${e.lngLat.lng.toFixed(4)}`;
        setComparePoint({
          lat: e.lngLat.lat,
          lng: e.lngLat.lng,
          address,
        });
        return;
      }

      // Explore mode: progressive disclosure, no mode switch
      if (feature) {
        selectFeature({
          layerId: feature.layer.id,
          properties: (feature.properties ?? {}) as Record<string, unknown>,
          coordinates: [e.lngLat.lng, e.lngLat.lat],
        });
        const featureAddress = feature?.properties?.address as string | undefined;
        setAnalysisPoint({
          lat: e.lngLat.lat,
          lng: e.lngLat.lng,
          ...(featureAddress !== undefined ? { address: featureAddress } : {}),
        });
      } else {
        selectFeature(null);
      }
    },
    [mode, selectFeature, setAnalysisPoint, setComparePoint],
  );

  return { handleFeatureClick };
}
