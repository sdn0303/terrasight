"use client";

import { Layer, Source } from "react-map-gl/maplibre";
import type { FeatureCollection } from "geojson";

interface Props {
  data: FeatureCollection;
  visible: boolean;
}

export function SchoolLayer({ data, visible }: Props) {
  if (!visible) return null;
  return (
    <Source id="schools" type="geojson" data={data}>
      <Layer
        id="schools-circle"
        type="circle"
        paint={{
          "circle-radius": 5,
          "circle-color": "#10b981",
          "circle-opacity": 0.9,
        }}
      />
    </Source>
  );
}
