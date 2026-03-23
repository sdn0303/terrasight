"use client";

import { Layer, Source } from "react-map-gl/maplibre";

interface Props {
  visible: boolean;
}

/**
 * Liquefaction risk layer (液状化危険度).
 *
 * Color mapping by PL index class (PL区分):
 * - 小（0≦PL≦5）  → low risk: yellow-green
 * - 中（5<PL≦15） → moderate risk: orange
 * - 大（PL>15）   → high risk: red
 *
 * Source: MLIT 液状化危険度評価結果 (point data)
 */
export function LiquefactionLayer({ visible }: Props) {
  if (!visible) return null;
  return (
    <Source
      id="liquefaction"
      type="geojson"
      data="/geojson/pl-liquefaction-tokyo.geojson"
    >
      <Layer
        id="liquefaction-circle"
        type="circle"
        paint={{
          "circle-radius": [
            "interpolate",
            ["linear"],
            ["zoom"],
            10,
            3,
            15,
            7,
          ],
          "circle-color": [
            "match",
            ["get", "PL区分"],
            "小（0≦PL≦5）",
            "#eab308",
            "中（5<PL≦15）",
            "#f97316",
            "大（PL>15）",
            "#ef4444",
            "#eab308",
          ] as unknown as string,
          "circle-opacity": 0.8,
          "circle-stroke-width": 1,
          "circle-stroke-color": "#0c0c14",
        }}
      />
    </Source>
  );
}
