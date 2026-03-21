"use client";

import type { FeatureCollection } from "geojson";
import { Layer, Source } from "react-map-gl/maplibre";

interface Props {
  data: FeatureCollection;
  visible: boolean;
}

export function FloodLayer({ data, visible }: Props) {
  if (!visible) return null;
  return (
    <Source id="flood" type="geojson" data={data}>
      <Layer
        id="flood-extrusion"
        type="fill-extrusion"
        paint={{
          "fill-extrusion-color": [
            "interpolate",
            ["linear"],
            ["get", "depth_rank"],
            0,
            "#1a6fff",
            2,
            "#ffd000",
            4,
            "#e04030",
          ],
          "fill-extrusion-height": ["*", ["get", "depth_rank"], 50],
          "fill-extrusion-base": 0,
          "fill-extrusion-opacity": 0.7,
        }}
      />
    </Source>
  );
}
