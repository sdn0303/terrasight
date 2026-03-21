"use client";

import { Layer, Source } from "react-map-gl/maplibre";

interface Props {
  visible: boolean;
}

export function VolcanoLayer({ visible }: Props) {
  if (!visible) return null;
  return (
    <Source id="volcano" type="geojson" data="/geojson/volcano-kanto.geojson">
      <Layer
        id="volcano-circle"
        type="circle"
        paint={{
          "circle-radius": 8,
          "circle-color": "#ef4444",
          "circle-stroke-width": 2,
          "circle-stroke-color": "#fbbf24",
          "circle-opacity": 0.9,
        }}
      />
    </Source>
  );
}
