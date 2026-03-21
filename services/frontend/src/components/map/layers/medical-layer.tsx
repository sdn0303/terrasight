"use client";

import type { FeatureCollection } from "geojson";
import { Layer, Source } from "react-map-gl/maplibre";

interface Props {
  data: FeatureCollection;
  visible: boolean;
}

export function MedicalLayer({ data, visible }: Props) {
  if (!visible) return null;
  return (
    <Source id="medical" type="geojson" data={data}>
      <Layer
        id="medical-circle"
        type="circle"
        paint={{
          "circle-radius": 5,
          "circle-color": "#6ee7b7",
          "circle-opacity": 0.9,
        }}
      />
    </Source>
  );
}
