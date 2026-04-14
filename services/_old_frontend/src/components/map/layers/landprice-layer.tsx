"use client";

import type { FeatureCollection } from "geojson";
import { Layer, Source } from "react-map-gl/maplibre";

interface Props {
  data: FeatureCollection;
  visible: boolean;
}

export function LandpriceLayer({ data, visible }: Props) {
  if (!visible) return null;
  return (
    <Source id="landprice" type="geojson" data={data}>
      <Layer
        id="landprice-circle"
        type="circle"
        paint={{
          "circle-radius": ["interpolate", ["linear"], ["zoom"], 10, 3, 15, 8],
          "circle-color": "#fbbf24",
          "circle-opacity": 0.85,
          "circle-stroke-width": 1,
          "circle-stroke-color": "#0c0c14",
        }}
      />
    </Source>
  );
}
