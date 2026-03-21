"use client";

import type { FeatureCollection } from "geojson";
import { useCallback, useMemo, useState } from "react";
import type { MapLayerMouseEvent } from "react-map-gl/maplibre";
import { ComparePanel } from "@/components/compare-panel";
import { DashboardStats } from "@/components/dashboard-stats";
import { LayerPanel } from "@/components/layer-panel";
import {
  AdminBoundaryLayer,
  DIDLayer,
  FaultLayer,
  FloodHistoryLayer,
  FloodLayer,
  GeologyLayer,
  LandformLayer,
  LandpriceLayer,
  LandslideLayer,
  MedicalLayer,
  ParkLayer,
  PopulationMeshLayer,
  SchoolDistrictLayer,
  SchoolLayer,
  SoilLayer,
  StationLayer,
  SteepSlopeLayer,
  TsunamiLayer,
  UrbanPlanLayer,
  VolcanoLayer,
  ZoningLayer,
} from "@/components/map/layers";
import { MapView } from "@/components/map/map-view";
import { PopupCard } from "@/components/map/popup-card";
import { YearSlider } from "@/components/map/year-slider";
import { ScoreCard } from "@/components/score-card/score-card";
import { StatusBar } from "@/components/status-bar";
import { useAreaData } from "@/features/area-data/api/use-area-data";
import { useHealth } from "@/features/health/api/use-health";
import { useMapUrlState } from "@/hooks/use-map-url-state";
import { LAYERS } from "@/lib/layers";
import { useMapStore } from "@/stores/map-store";
import { useUIStore } from "@/stores/ui-store";

const EMPTY_FC: FeatureCollection = { type: "FeatureCollection", features: [] };

/**
 * Registry mapping layer IDs to their React components.
 * Static layers receive { visible }, API layers receive { data, visible }.
 * PopulationMeshLayer additionally receives { selectedYear }.
 *
 * This eliminates the manual enumeration DRY violation (14 → 21 layers)
 * while keeping individual layer components for custom paint expressions.
 */
const STATIC_LAYER_COMPONENTS: Record<
  string,
  React.ComponentType<{ visible: boolean } & Record<string, unknown>>
> = {
  did: DIDLayer,
  landform: LandformLayer,
  geology: GeologyLayer,
  admin_boundary: AdminBoundaryLayer,
  fault: FaultLayer,
  flood_history: FloodHistoryLayer,
  soil: SoilLayer,
  volcano: VolcanoLayer,
  station: StationLayer,
  school_district: SchoolDistrictLayer,
  landslide: LandslideLayer,
  park: ParkLayer,
  tsunami: TsunamiLayer,
  urban_plan: UrbanPlanLayer,
};

const API_LAYER_COMPONENTS: Record<
  string,
  React.ComponentType<{ data: FeatureCollection; visible: boolean }>
> = {
  landprice: LandpriceLayer,
  flood: FloodLayer,
  steep_slope: SteepSlopeLayer,
  schools: SchoolLayer,
  medical: MedicalLayer,
  zoning: ZoningLayer,
};

export default function Home() {
  useMapUrlState();
  const { viewState, visibleLayers, selectFeature, getBBox } = useMapStore();
  const { compareMode, setComparePoint } = useUIStore();
  const [bbox, setBbox] = useState(() => getBBox());
  const [populationYear, setPopulationYear] = useState(2020);

  const layers = useMemo(() => [...visibleLayers], [visibleLayers]);
  const { data: areaData, isLoading } = useAreaData(bbox, layers);
  const { data: health } = useHealth();

  // Derive popup config for click-inspect
  const selectedFeature = useMapStore((s) => s.selectedFeature);
  const selectedLayerConfig = useMemo(() => {
    if (!selectedFeature) return null;
    // Match by layer ID prefix (e.g., "landprice-circle" → "landprice")
    return LAYERS.find((l) => selectedFeature.layerId.startsWith(l.id)) ?? null;
  }, [selectedFeature]);

  const handleMoveEnd = useCallback(() => {
    setBbox(getBBox());
  }, [getBBox]);

  const handleFeatureClick = useCallback(
    (e: MapLayerMouseEvent) => {
      const feature = e.features?.[0];
      if (compareMode) {
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

  // Separate static and API layers from config
  const staticLayers = useMemo(
    () => LAYERS.filter((l) => l.source === "static"),
    [],
  );
  const apiLayers = useMemo(() => LAYERS.filter((l) => l.source === "api"), []);

  return (
    <div className="relative h-screen w-screen overflow-hidden">
      <LayerPanel />
      <ScoreCard />

      <MapView onMoveEnd={handleMoveEnd} onFeatureClick={handleFeatureClick}>
        {/* API-driven layers: receive data from useAreaData */}
        {apiLayers.map((layer) => {
          const Component = API_LAYER_COMPONENTS[layer.id];
          if (!Component) return null;
          const layerData =
            areaData != null
              ? ((areaData as Record<string, unknown>)[layer.id] as
                  | FeatureCollection
                  | undefined)
              : undefined;
          return (
            <Component
              key={layer.id}
              data={layerData ?? EMPTY_FC}
              visible={visibleLayers.has(layer.id)}
            />
          );
        })}

        {/* Static layers: load GeoJSON from /geojson/ on mount */}
        {staticLayers.map((layer) => {
          // PopulationMeshLayer needs special handling for year slider
          if (layer.id === "population_mesh") {
            return (
              <PopulationMeshLayer
                key={layer.id}
                visible={visibleLayers.has(layer.id)}
                selectedYear={populationYear}
              />
            );
          }
          const Component = STATIC_LAYER_COMPONENTS[layer.id];
          if (!Component) return null;
          return (
            <Component key={layer.id} visible={visibleLayers.has(layer.id)} />
          );
        })}

        {/* Year slider for population mesh — only visible when layer is active */}
        <YearSlider
          value={populationYear}
          onChange={setPopulationYear}
          visible={visibleLayers.has("population_mesh")}
        />
      </MapView>

      {/* Click-inspect popup */}
      {selectedFeature && selectedLayerConfig?.popupFields && (
        <div
          className="fixed z-30 pointer-events-none"
          style={{
            top: "50%",
            left: "50%",
            transform: "translate(-50%, -50%)",
          }}
        >
          <div className="pointer-events-auto">
            <PopupCard
              layerNameJa={selectedLayerConfig.nameJa}
              fields={selectedLayerConfig.popupFields}
              properties={selectedFeature.properties}
            />
          </div>
        </div>
      )}

      <ComparePanel />
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
