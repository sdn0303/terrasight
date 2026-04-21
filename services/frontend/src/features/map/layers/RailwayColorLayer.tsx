"use client";

import type { FeatureCollection } from "geojson";
import { useMemo } from "react";
import { Layer, Source } from "react-map-gl/mapbox";
import { DEFAULT_COLOR, RAILWAY_COLORS } from "@/lib/railway-colors";

interface RailwayColorLayerProps {
  data: FeatureCollection;
  visible: boolean;
}

/**
 * 乗降客数タブ: 路線テーマカラーで鉄道路線を色分け表示。
 * Ref: DESIGN.md Sec 6.4 — 乗降客数タブ, Sec 7 (Railway Colors)
 */
export function RailwayColorLayer({ data, visible }: RailwayColorLayerProps) {
  const colorExpression = useMemo(() => {
    const cases: (string | ["get", string])[] = [];
    for (const [key, color] of Object.entries(RAILWAY_COLORS)) {
      cases.push(key, color);
    }
    return [
      "match",
      ["concat", ["get", "operator_name"], "/", ["get", "line_name"]],
      ...cases,
      DEFAULT_COLOR,
    ] as unknown as mapboxgl.Expression;
  }, []);

  return (
    <Source id="railway-color-source" type="geojson" data={data}>
      <Layer
        id="railway-colored-line"
        type="line"
        layout={{ visibility: visible ? "visible" : "none" }}
        paint={{
          "line-color": colorExpression,
          "line-width": ["interpolate", ["linear"], ["zoom"], 8, 1.5, 14, 3],
          "line-opacity": 0.85,
        }}
        minzoom={8}
      />
    </Source>
  );
}
