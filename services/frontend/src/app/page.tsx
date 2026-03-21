"use client";

import { useCallback, useMemo, useRef } from "react";
import type { FeatureCollection } from "geojson";
import type { MapLayerMouseEvent } from "react-map-gl/maplibre";
import { MapView } from "@/components/map/map-view";
import {
  FloodLayer,
  LandpriceLayer,
  MedicalLayer,
  SchoolLayer,
  SteepSlopeLayer,
  ZoningLayer,
} from "@/components/map/layers";
import { CRTOverlay } from "@/components/crt-overlay";
import { LayerPanel } from "@/components/layer-panel";
import { StatusBar } from "@/components/status-bar";
import { useAreaData } from "@/features/area-data/api/use-area-data";
import { useHealth } from "@/features/health/api/use-health";
import { useMapStore } from "@/stores/map-store";

const EMPTY_FC: FeatureCollection = { type: "FeatureCollection", features: [] };

export default function Home() {
  const { viewState, visibleLayers, selectFeature, getBBox } = useMapStore();
  const bboxRef = useRef(getBBox());

  const layers = useMemo(() => [...visibleLayers], [visibleLayers]);
  const { data: areaData, isLoading } = useAreaData(bboxRef.current, layers);
  const { data: health } = useHealth();

  const handleMoveEnd = useCallback(() => {
    bboxRef.current = getBBox();
  }, [getBBox]);

  const handleFeatureClick = useCallback(
    (e: MapLayerMouseEvent) => {
      const feature = e.features?.[0];
      if (feature) {
        selectFeature({
          layerId: feature.layer.id,
          properties: feature.properties as Record<string, unknown>,
          coordinates: [e.lngLat.lng, e.lngLat.lat],
        });
      } else {
        selectFeature(null);
      }
    },
    [selectFeature],
  );

  const isDemoMode = health ? !health.reinfolib_key_set : true;

  return (
    <div className="relative h-screen w-screen overflow-hidden">
      <LayerPanel />

      <MapView onMoveEnd={handleMoveEnd} onFeatureClick={handleFeatureClick}>
        <LandpriceLayer
          data={(areaData?.landprice as FeatureCollection | undefined) ?? EMPTY_FC}
          visible={visibleLayers.has("landprice")}
        />
        <ZoningLayer
          data={(areaData?.zoning as FeatureCollection | undefined) ?? EMPTY_FC}
          visible={visibleLayers.has("zoning")}
        />
        <FloodLayer
          data={(areaData?.flood as FeatureCollection | undefined) ?? EMPTY_FC}
          visible={visibleLayers.has("flood")}
        />
        <SteepSlopeLayer
          data={(areaData?.steep_slope as FeatureCollection | undefined) ?? EMPTY_FC}
          visible={visibleLayers.has("steep_slope")}
        />
        <SchoolLayer
          data={(areaData?.schools as FeatureCollection | undefined) ?? EMPTY_FC}
          visible={visibleLayers.has("schools")}
        />
        <MedicalLayer
          data={(areaData?.medical as FeatureCollection | undefined) ?? EMPTY_FC}
          visible={visibleLayers.has("medical")}
        />
      </MapView>

      <CRTOverlay />
      <StatusBar
        lat={viewState.latitude}
        lng={viewState.longitude}
        zoom={viewState.zoom}
        isLoading={isLoading}
        isDemoMode={isDemoMode}
      />
    </div>
  );
}
