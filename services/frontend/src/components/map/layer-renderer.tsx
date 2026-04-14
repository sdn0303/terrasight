"use client";

import type { FeatureCollection } from "geojson";
import { useMemo } from "react";
import { AreaHighlight } from "@/components/map/area-highlight";
import { LandPriceYearSlider } from "@/components/map/land-price-year-slider";
import {
  AdminBoundaryLayer,
  DIDLayer,
  FaultLayer,
  FloodHistoryLayer,
  FloodLayer,
  GeologyLayer,
  LandformLayer,
  LandPriceExtrusionLayer,
  LandpriceLayer,
  LandslideLayer,
  LiquefactionLayer,
  MedicalLayer,
  PopulationMeshLayer,
  RailwayLayer,
  SchoolLayer,
  SeismicLayer,
  SoilLayer,
  StationLayer,
  SteepSlopeLayer,
  VolcanoLayer,
  ZoningLayer,
} from "@/components/map/layers";
import { BoundaryLayer } from "@/components/map/layers/boundary-layer";
import { YearSlider } from "@/components/map/year-slider";
import { useThemeLayers } from "@/hooks/use-theme-layers";
import { useVisibleStaticLayers } from "@/hooks/use-visible-static-layers";
import { canonicalLayerId } from "@/lib/layer-ids";
import type { LayerConfig } from "@/lib/layers";

const EMPTY_FC: FeatureCollection = {
  type: "FeatureCollection",
  features: [],
};

const STATIC_LAYER_COMPONENTS: Record<
  string,
  React.ComponentType<
    { visible: boolean; data?: FeatureCollection } & Record<string, unknown>
  >
> = {
  did: DIDLayer,
  landform: LandformLayer,
  geology: GeologyLayer,
  admin_boundary: AdminBoundaryLayer,
  fault: FaultLayer,
  flood_history: FloodHistoryLayer,
  liquefaction: LiquefactionLayer,
  railway: RailwayLayer,
  seismic: SeismicLayer,
  soil: SoilLayer,
  volcano: VolcanoLayer,
  station: StationLayer,
  landslide: LandslideLayer,
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

interface LayerRendererProps {
  visibleLayers: Set<string>;
  staticLayers: LayerConfig[];
  apiLayers: LayerConfig[];
  areaData: Record<string, unknown> | null;
  landPriceData: FeatureCollection;
  isLandPriceFetching: boolean;
  isLandPriceError: boolean;
  isZoomTooLow: boolean;
  populationYear: number;
  setPopulationYear: (year: number) => void;
  landPriceYear: number;
  setLandPriceYear: (year: number | null) => void;
  landPriceFeatureCount?: number;
}

export function LayerRenderer({
  visibleLayers,
  staticLayers,
  apiLayers,
  areaData,
  landPriceData,
  isLandPriceFetching,
  isLandPriceError,
  isZoomTooLow,
  populationYear,
  setPopulationYear,
  landPriceYear,
  setLandPriceYear,
  landPriceFeatureCount,
}: LayerRendererProps) {
  const { visibleLayerIds } = useThemeLayers();

  // A layer is visible if the map-store toggles it OR the active theme includes it.
  const isVisible = (id: string) =>
    visibleLayers.has(id) || visibleLayerIds.has(id);

  // Compute visible static layer IDs for batched hook
  const visibleStaticIds = useMemo(
    () =>
      staticLayers
        .filter((l) => isVisible(l.id) && l.id !== "population_mesh")
        .map((l) => l.id),
    [staticLayers, visibleLayers, visibleLayerIds],
  );

  // Single batched query for all visible static layers
  const staticLayerData = useVisibleStaticLayers(visibleStaticIds);

  return (
    <>
      <BoundaryLayer />
      <AreaHighlight />

      <LandPriceExtrusionLayer
        data={landPriceData}
        visible={isVisible("land_price_ts")}
        selectedYear={landPriceYear}
        isFetching={isLandPriceFetching}
      />

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
            visible={isVisible(layer.id)}
          />
        );
      })}

      {staticLayers.map((layer) => {
        if (layer.id === "population_mesh") {
          return (
            <PopulationMeshLayer
              key={layer.id}
              visible={isVisible(layer.id)}
              selectedYear={populationYear}
            />
          );
        }
        const Component = STATIC_LAYER_COMPONENTS[layer.id];
        if (!Component) return null;
        const layerData = staticLayerData.get(canonicalLayerId(layer.id));
        return (
          <Component
            key={layer.id}
            visible={isVisible(layer.id)}
            {...(layerData !== undefined && { data: layerData })}
          />
        );
      })}

      <YearSlider
        value={populationYear}
        onChange={setPopulationYear}
        visible={isVisible("population_mesh")}
      />

      <LandPriceYearSlider
        value={landPriceYear}
        onChange={setLandPriceYear}
        visible={isVisible("land_price_ts")}
        isFetching={isLandPriceFetching}
        isError={isLandPriceError}
        isZoomTooLow={isZoomTooLow}
        {...(landPriceFeatureCount !== undefined
          ? { featureCount: landPriceFeatureCount }
          : {})}
      />
    </>
  );
}
