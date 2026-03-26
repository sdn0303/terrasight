"use client";

import type { FeatureCollection } from "geojson";
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
  ParkLayer,
  PopulationMeshLayer,
  RailwayLayer,
  SchoolDistrictLayer,
  SchoolLayer,
  SeismicLayer,
  SoilLayer,
  StationLayer,
  SteepSlopeLayer,
  TsunamiLayer,
  UrbanPlanLayer,
  VolcanoLayer,
  ZoningLayer,
} from "@/components/map/layers";
import { BoundaryLayer } from "@/components/map/layers/boundary-layer";
import { YearSlider } from "@/components/map/year-slider";
import type { LayerConfig } from "@/lib/layers";

const EMPTY_FC: FeatureCollection = {
  type: "FeatureCollection",
  features: [],
};

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
  liquefaction: LiquefactionLayer,
  railway: RailwayLayer,
  seismic: SeismicLayer,
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
  return (
    <>
      <BoundaryLayer />
      <AreaHighlight />

      <LandPriceExtrusionLayer
        data={landPriceData}
        visible={visibleLayers.has("land_price_ts")}
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
            visible={visibleLayers.has(layer.id)}
          />
        );
      })}

      {staticLayers.map((layer) => {
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

      <YearSlider
        value={populationYear}
        onChange={setPopulationYear}
        visible={visibleLayers.has("population_mesh")}
      />

      <LandPriceYearSlider
        value={landPriceYear}
        onChange={setLandPriceYear}
        visible={visibleLayers.has("land_price_ts")}
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
