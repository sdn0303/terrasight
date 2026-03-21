"use client";

import { Layer, Source } from "react-map-gl/maplibre";
import type { FeatureCollection } from "geojson";

interface Props {
  data: FeatureCollection;
  visible: boolean;
}

export function SteepSlopeLayer({ data, visible }: Props) {
  if (!visible) return null;
  return (
    <Source id="steep_slope" type="geojson" data={data}>
      <Layer
        id="steep-slope-extrusion"
        type="fill-extrusion"
        paint={{
          "fill-extrusion-color": "#e04030",
          "fill-extrusion-height": 100,
          "fill-extrusion-base": 0,
          "fill-extrusion-opacity": 0.6,
        }}
      />
    </Source>
  );
}
