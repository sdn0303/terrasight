"use client";

import { Source, Layer } from "react-map-gl/maplibre";
import type { ActiveLayers } from "@/lib/layers";

interface MapLayersProps {
  layerData: Record<string, GeoJSON.FeatureCollection>;
  activeLayers: ActiveLayers;
}

export default function MapLayers({ layerData, activeLayers }: MapLayersProps) {
  return (
    <>
      {activeLayers.landprice && layerData.landprice && (
        <Source id="landprice-source" type="geojson" data={layerData.landprice}>
          <Layer
            id="landprice-circle"
            type="circle"
            paint={{
              "circle-radius": ["interpolate", ["linear"], ["zoom"], 10, 3, 15, 8],
              "circle-color": "#22d3ee",
              "circle-opacity": 0.8,
              "circle-stroke-width": 1,
              "circle-stroke-color": "#0a0a0f",
            }}
          />
        </Source>
      )}
    </>
  );
}
