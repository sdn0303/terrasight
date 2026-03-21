"use client";

import { Layer, Source } from "react-map-gl/maplibre";

interface Props {
  visible: boolean;
}

export function StationLayer({ visible }: Props) {
  if (!visible) return null;
  return (
    <Source id="station" type="geojson" data="/geojson/station-tokyo.geojson">
      <Layer
        id="station-circle"
        type="circle"
        paint={{
          "circle-radius": ["interpolate", ["linear"], ["zoom"], 10, 3, 15, 8],
          "circle-color": "#f472b6",
          "circle-opacity": 0.85,
          "circle-stroke-width": 1,
          "circle-stroke-color": "#0c0c14",
        }}
      />
    </Source>
  );
}
