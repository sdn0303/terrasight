"use client";

import { useCallback } from "react";
import type { MapLayerMouseEvent } from "react-map-gl/maplibre";
import { useMapStore } from "@/stores/map-store";
import { useUIStore } from "@/stores/ui-store";

export function useMapInteraction() {
  const selectFeature = useMapStore((s) => s.selectFeature);
  const selectArea = useMapStore((s) => s.selectArea);
  const setAnalysisPoint = useMapStore((s) => s.setAnalysisPoint);
  const mode = useUIStore((s) => s.mode);
  const addComparePoint = useUIStore((s) => s.addComparePoint);
  const setInsight = useUIStore((s) => s.setInsight);

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
        addComparePoint({
          lat: e.lngLat.lat,
          lng: e.lngLat.lng,
          address,
        });
        return;
      }

      // Explore mode: progressive disclosure, no mode switch
      if (feature) {
        // Check if clicked an admin boundary feature
        const layerId = feature.layer.id;
        if (
          layerId === "admin-boundary-fill" ||
          layerId === "admin-boundary-line"
        ) {
          const props = feature.properties ?? {};
          selectArea({
            code: (props.adminCode as string) ?? "",
            name:
              (props.cityName as string) ?? (props.prefName as string) ?? "",
            level: (props.cityName as string) ? "municipality" : "prefecture",
            bbox: { south: 0, west: 0, north: 0, east: 0 }, // TODO: compute from geometry
          });
          return;
        }

        selectFeature({
          layerId: feature.layer.id,
          properties: (feature.properties ?? {}) as Record<string, unknown>,
          coordinates: [e.lngLat.lng, e.lngLat.lat],
        });
        const featureAddress = feature?.properties?.address as
          | string
          | undefined;
        setAnalysisPoint({
          lat: e.lngLat.lat,
          lng: e.lngLat.lng,
          ...(featureAddress !== undefined ? { address: featureAddress } : {}),
        });
        // Open the Insight drawer for this point
        setInsight({
          kind: "point",
          lat: e.lngLat.lat,
          lng: e.lngLat.lng,
        });
      } else {
        selectFeature(null);
      }
    },
    [
      mode,
      selectArea,
      selectFeature,
      setAnalysisPoint,
      addComparePoint,
      setInsight,
    ],
  );

  return { handleFeatureClick };
}
