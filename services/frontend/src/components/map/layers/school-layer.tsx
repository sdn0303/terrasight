"use client";

import type { FeatureCollection } from "geojson";
import { Layer, Source } from "react-map-gl/mapbox";

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
          "circle-color": "#34d399",
          "circle-opacity": 0.9,
        }}
      />
    </Source>
  );
}
