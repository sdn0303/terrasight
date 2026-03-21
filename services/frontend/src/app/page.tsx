"use client";

import { useCallback, useMemo, useState } from "react";
import type { FeatureCollection } from "geojson";
import type { MapLayerMouseEvent } from "react-map-gl/maplibre";
import { ComparePanel } from "@/components/compare-panel";
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
import { DashboardStats } from "@/components/dashboard-stats";
import { LayerPanel } from "@/components/layer-panel";
import { ScoreCard } from "@/components/score-card/score-card";
import { StatusBar } from "@/components/status-bar";
import { useAreaData } from "@/features/area-data/api/use-area-data";
import { useHealth } from "@/features/health/api/use-health";
import { useMapUrlState } from "@/hooks/use-map-url-state";
import { useMapStore } from "@/stores/map-store";
import { useUIStore } from "@/stores/ui-store";

const EMPTY_FC: FeatureCollection = { type: "FeatureCollection", features: [] };

export default function Home() {
  useMapUrlState();
  const { viewState, visibleLayers, selectFeature, getBBox } = useMapStore();
  const { compareMode, setComparePoint } = useUIStore();
  const [bbox, setBbox] = useState(() => getBBox());

  const layers = useMemo(() => [...visibleLayers], [visibleLayers]);
  const { data: areaData, isLoading } = useAreaData(bbox, layers);
  const { data: health } = useHealth();

  const handleMoveEnd = useCallback(() => {
    setBbox(getBBox());
  }, [getBBox]);

  const handleFeatureClick = useCallback(
    (e: MapLayerMouseEvent) => {
      const feature = e.features?.[0];
      if (compareMode) {
        // In compare mode, set compare points instead of selecting a feature
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
      } else if (feature) {
        selectFeature({
          layerId: feature.layer.id,
          properties: (feature.properties ?? {}) as Record<string, unknown>,
          coordinates: [e.lngLat.lng, e.lngLat.lat],
        });
      } else {
        selectFeature(null);
      }
    },
    [compareMode, selectFeature, setComparePoint],
  );

  const isDemoMode = health ? !health.reinfolib_key_set : true;

  return (
    <div className="relative h-screen w-screen overflow-hidden">
      <LayerPanel />
      <ScoreCard />

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

      <ComparePanel />
      <CRTOverlay />
      <StatusBar
        lat={viewState.latitude}
        lng={viewState.longitude}
        zoom={viewState.zoom}
        isLoading={isLoading}
        isDemoMode={isDemoMode}
      />
      <DashboardStats />
    </div>
  );
}
