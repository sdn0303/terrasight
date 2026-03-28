"use client";

import type { FeatureCollection } from "geojson";
import { Layer, Source } from "react-map-gl/maplibre";
import { useStaticLayer } from "@/hooks/use-static-layer";

interface Props {
  visible: boolean;
  data?: FeatureCollection;
}

const GEOLOGY_COLORS: [string, string][] = [
  ["第四紀堆積岩類", "#fbbf24"],
  ["第三紀堆積岩類", "#f97316"],
  ["火山岩類", "#ef4444"],
  ["深成岩類", "#ec4899"],
  ["変成岩類", "#8b5cf6"],
  ["古生代堆積岩類", "#6366f1"],
  ["中生代堆積岩類", "#3b82f6"],
  ["埋立地", "#a855f7"],
  ["水域", "#06b6d4"],
];

export function GeologyLayer({ visible, data: propData }: Props) {
  const selfFetch = useStaticLayer("13", "geology", visible && !propData);
  const data = propData ?? selfFetch.data;
  if (!visible || !data) return null;

  const colorExpr: unknown[] = ["match", ["get", "geologyCategory2"]];
  for (const [category, color] of GEOLOGY_COLORS) {
    colorExpr.push(category, color);
  }
  colorExpr.push("#71717a"); // fallback

  return (
    <Source id="geology" type="geojson" data={data}>
      <Layer
        id="geology-fill"
        type="fill"
        paint={{
          "fill-color": colorExpr as unknown as string,
          "fill-opacity": 0.5,
        }}
      />
      <Layer
        id="geology-outline"
        type="line"
        paint={{
          "line-color": "rgba(255, 255, 255, 0.3)",
          "line-width": 0.5,
        }}
      />
    </Source>
  );
}
