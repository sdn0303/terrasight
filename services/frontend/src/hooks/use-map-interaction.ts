"use client";

import { useCallback } from "react";
import type { MapMouseEvent } from "react-map-gl/mapbox";
import { useMapStore } from "@/stores/map-store";
import { useUIStore } from "@/stores/ui-store";

export function useMapInteraction() {
  const selectFeature = useMapStore((s) => s.selectFeature);
  const selectArea = useMapStore((s) => s.selectArea);
  const setAnalysisPoint = useMapStore((s) => s.setAnalysisPoint);
  const setSelectedArea = useUIStore((s) => s.setSelectedArea);

  const handleFeatureClick = useCallback(
    (e: MapMouseEvent) => {
      const feature = e.features?.[0];

      if (feature) {
        const layerId = feature.layer?.id;
        if (
          layerId === "admin-boundary-fill" ||
          layerId === "admin-boundary-line"
        ) {
          const props = feature.properties ?? {};
          const area = {
            code: (props.adminCode as string) ?? "",
            name:
              (props.cityName as string) ?? (props.prefName as string) ?? "",
            level: ((props.cityName as string)
              ? "municipality"
              : "prefecture") as "prefecture" | "municipality",
            bbox: { south: 0, west: 0, north: 0, east: 0 },
          };
          selectArea(area);
          setSelectedArea({
            code: area.code,
            name: area.name,
            level: area.level,
            lat: e.lngLat.lat,
            lng: e.lngLat.lng,
          });
          return;
        }

        selectFeature({
          layerId: feature.layer?.id ?? "",
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
        setSelectedArea({
          code: "",
          name: featureAddress ?? "",
          level: "municipality",
          lat: e.lngLat.lat,
          lng: e.lngLat.lng,
        });
      } else {
        selectFeature(null);
      }
    },
    [selectArea, selectFeature, setAnalysisPoint, setSelectedArea],
  );

  return { handleFeatureClick };
}
