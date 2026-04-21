"use client";

import type { FeatureCollection } from "geojson";
import { Layer, Source } from "react-map-gl/mapbox";

interface YieldLayerProps {
  data: FeatureCollection;
  visible: boolean;
}

/**
 * 利回りタブ: 市区町村ポリゴンを利回り帯で3段階塗り分け。
 * Ref: DESIGN.md Sec 6.4 — 利回りタブ
 */
export function YieldLayer({ data, visible }: YieldLayerProps) {
  return (
    <Source id="yield-source" type="geojson" data={data}>
      <Layer
        id="yield-fill"
        type="fill"
        layout={{ visibility: visible ? "visible" : "none" }}
        paint={{
          "fill-color": [
            "step",
            ["coalesce", ["get", "yield_pct"], 0],
            "#FBBF24",
            4,
            "#3B82F6",
            6,
            "#22C55E",
          ],
          "fill-opacity": 0.45,
        }}
      />
      <Layer
        id="yield-outline"
        type="line"
        layout={{ visibility: visible ? "visible" : "none" }}
        paint={{
          "line-color": "#FFFFFF1A",
          "line-width": 1,
        }}
      />
    </Source>
  );
}
