"use client";

import { Layer, Source } from "react-map-gl/maplibre";

interface Props {
  visible: boolean;
}

const LANDFORM_COLORS: [string, string][] = [
  ["山地", "#6b7280"],
  ["火山地", "#ef4444"],
  ["丘陵地", "#84cc16"],
  ["台地", "#eab308"],
  ["低地", "#3b82f6"],
  ["埋立地", "#a855f7"],
  ["水面", "#06b6d4"],
];

export function LandformLayer({ visible }: Props) {
  if (!visible) return null;

  const colorExpr: unknown[] = ["match", ["get", "landformCategory"]];
  for (const [category, color] of LANDFORM_COLORS) {
    colorExpr.push(category, color);
  }
  colorExpr.push("#71717a"); // fallback

  return (
    <Source id="landform" type="geojson" data="/geojson/landform-tokyo.geojson">
      <Layer
        id="landform-fill"
        type="fill"
        paint={{
          "fill-color": colorExpr as unknown as string,
          "fill-opacity": 0.5,
        }}
      />
      <Layer
        id="landform-outline"
        type="line"
        paint={{
          "line-color": "rgba(255, 255, 255, 0.3)",
          "line-width": 0.5,
        }}
      />
    </Source>
  );
}
