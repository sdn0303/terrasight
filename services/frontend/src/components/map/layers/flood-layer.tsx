"use client";

import type { FeatureCollection } from "geojson";
import { Layer, Source } from "react-map-gl/maplibre";

interface Props {
  data: FeatureCollection;
  visible: boolean;
}

/**
 * MLIT depth_rank is a text string like "0.5m未満", "0.5-3.0m", "3.0-5.0m", "5.0m以上".
 * We use match expressions to map these to numeric values for extrusion height and color.
 *
 *   depth_rank text    → numeric rank (0-4)
 *   "0.5m未満"         → 1
 *   "0.5-3.0m"         → 2
 *   "3.0-5.0m"         → 3
 *   "5.0m以上"         → 4
 *   (other)            → 1 (fallback)
 */
const DEPTH_RANK_NUMERIC = [
  "match",
  ["get", "depth_rank"],
  "0.5m未満",
  1,
  "0.5-3.0m",
  2,
  "3.0-5.0m",
  3,
  "5.0m以上",
  4,
  1, // fallback
] as unknown as maplibregl.ExpressionSpecification;

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
            DEPTH_RANK_NUMERIC,
            0,
            "#1a6fff",
            2,
            "#ffd000",
            4,
            "#e04030",
          ],
          "fill-extrusion-height": ["*", DEPTH_RANK_NUMERIC, 50],
          "fill-extrusion-base": 0,
          "fill-extrusion-opacity": 0.7,
        }}
      />
    </Source>
  );
}
