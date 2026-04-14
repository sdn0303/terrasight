"use client";

import type { FeatureCollection } from "geojson";
import { Layer, Source } from "react-map-gl/mapbox";

interface Props {
  data: FeatureCollection;
  visible: boolean;
}

/**
 * depth_rank is a numeric value from the backend:
 *   0 = outside flood zone
 *   1 = < 0.5 m
 *   2 = 0.5 – 3 m
 *   3 = 3 – 5 m
 *   4 = 5 – 10 m
 *   5 = ≥ 10 m
 *
 * The value is used directly in interpolate expressions for color and extrusion height.
 */
const DEPTH_RANK_EXPR = [
  "get",
  "depth_rank",
] as unknown as ["get", string];

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
            DEPTH_RANK_EXPR,
            0,
            "#1a6fff",
            2,
            "#ffd000",
            5,
            "#e04030",
          ],
          "fill-extrusion-height": ["*", DEPTH_RANK_EXPR, 50],
          "fill-extrusion-base": 0,
          "fill-extrusion-opacity": 0.7,
        }}
      />
    </Source>
  );
}
