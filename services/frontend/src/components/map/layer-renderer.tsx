"use client";

import type { FeatureCollection } from "geojson";
import { AreaHighlight } from "@/components/map/area-highlight";
import {
  AdminBoundaryLayer,
  DIDLayer,
  FaultLayer,
  FloodHistoryLayer,
  FloodLayer,
  GeologyLayer,
  LandformLayer,
  LandPriceExtrusionLayer,
  LandPricePolygonLayer,
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
  TransactionPolygonLayer,
  VolcanoLayer,
  ZoningLayer,
} from "@/components/map/layers";
import { BoundaryLayer } from "@/components/map/layers/boundary-layer";
import { useLandPriceAggregation } from "@/features/land-prices/api/use-land-price-aggregation";
import { useTransactionAggregation } from "@/features/transactions/api/use-transaction-aggregation";
import { useThemeLayers } from "@/hooks/use-theme-layers";
import { useVisibleStaticLayers } from "@/hooks/use-visible-static-layers";
import type { AreaDataResponse } from "@/lib/api/schemas/area-data";
import type { LandPriceAggregation } from "@/lib/api/schemas/land-price-aggregation";
import type { TransactionAggregation } from "@/lib/api/schemas/transaction-aggregation";
import { canonicalLayerId } from "@/lib/layer-ids";
import type { LayerConfig } from "@/lib/layers";

const EMPTY_FC: FeatureCollection = {
  type: "FeatureCollection",
  features: [],
};

const EMPTY_LAND_PRICE_AGG: LandPriceAggregation = {
  type: "FeatureCollection",
  features: [],
};

const EMPTY_TRANSACTION_AGG: TransactionAggregation = {
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
  areaData: AreaDataResponse | null;
  landPriceData: FeatureCollection;
  isLandPriceFetching: boolean;
  populationYear: number;
  landPriceYear: number;
}

export function LayerRenderer({
  visibleLayers,
  staticLayers,
  apiLayers,
  areaData,
  landPriceData,
  isLandPriceFetching,
  populationYear,
  landPriceYear,
}: LayerRendererProps) {
  const { visibleLayerIds } = useThemeLayers();
  const { data: landPriceAggData } = useLandPriceAggregation();
  const { data: transactionAggData } = useTransactionAggregation();

  // A layer is visible if the map-store toggles it OR the active theme includes it.
  const isVisible = (id: string) =>
    visibleLayers.has(id) || visibleLayerIds.has(id);

  // Compute visible static layer IDs for batched hook
  const visibleStaticIds = staticLayers
    .filter((l) => isVisible(l.id) && l.id !== "population_mesh")
    .map((l) => l.id);

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

      <LandPricePolygonLayer
        data={landPriceAggData ?? EMPTY_LAND_PRICE_AGG}
        visible={isVisible("land_price_polygon")}
      />

      <TransactionPolygonLayer
        data={transactionAggData ?? EMPTY_TRANSACTION_AGG}
        visible={isVisible("transaction_polygon")}
      />

      {apiLayers.map((layer) => {
        const Component = API_LAYER_COMPONENTS[layer.id];
        if (!Component) return null;
        const layerData =
          areaData != null
            ? (areaData[layer.id as keyof AreaDataResponse] as
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
    </>
  );
}
