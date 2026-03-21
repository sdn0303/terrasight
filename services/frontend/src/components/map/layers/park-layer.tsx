"use client";

import { Layer, Source } from "react-map-gl/maplibre";

interface Props {
  visible: boolean;
}

export function ParkLayer({ visible }: Props) {
  if (!visible) return null;
  return (
    <Source id="park" type="geojson" data="/geojson/park-tokyo.geojson">
      <Layer
        id="park-fill"
        type="fill"
        paint={{
          "fill-color": "#86efac",
          "fill-opacity": 0.25,
        }}
      />
      <Layer
        id="park-outline"
        type="line"
        paint={{
          "line-color": "#86efac",
          "line-width": 1,
          "line-opacity": 0.5,
        }}
      />
    </Source>
  );
}
