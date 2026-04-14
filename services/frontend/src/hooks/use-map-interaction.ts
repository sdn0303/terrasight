"use client";

import { useCallback } from "react";
import type { MapMouseEvent } from "react-map-gl/mapbox";
import { useMapStore } from "@/stores/map-store";
import { useUIStore } from "@/stores/ui-store";

export function useMapInteraction() {
  const selectFeature = useMapStore((s) => s.selectFeature);
  const selectArea = useMapStore((s) => s.selectArea);
  const setAnalysisPoint = useMapStore((s) => s.setAnalysisPoint);
  const setInsight = useUIStore((s) => s.setInsight);

  const handleFeatureClick = useCallback(
    (e: MapMouseEvent) => {
      const feature = e.features?.[0];

      // Progressive disclosure: click opens the insight drawer for the
      // feature. Pre-Phase-6 this branched on an `explore` vs `compare`
      // app mode; compare is now driven by the OpportunitiesSheet
      // checkbox column, so map clicks always take the drawer path.
      if (feature) {
        const layerId = feature.layer?.id;
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
        setInsight({
          kind: "point",
          lat: e.lngLat.lat,
          lng: e.lngLat.lng,
        });
      } else {
        selectFeature(null);
      }
    },
    [selectArea, selectFeature, setAnalysisPoint, setInsight],
  );

  return { handleFeatureClick };
}
