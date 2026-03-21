"use client";

import type { FeatureCollection } from "geojson";
import { Layer, Source } from "react-map-gl/maplibre";

interface Props {
  data: FeatureCollection;
  visible: boolean;
}

export function ZoningLayer({ data, visible }: Props) {
  if (!visible) return null;
  return (
    <Source id="zoning" type="geojson" data={data}>
      <Layer
        id="zoning-fill"
        type="fill"
        paint={{
          "fill-color": [
            "match",
            ["get", "zone_type"],
            "第一種低層住居専用地域",
            "#2563eb",
            "第二種低層住居専用地域",
            "#3b82f6",
            "第一種中高層住居専用地域",
            "#60a5fa",
            "第二種中高層住居専用地域",
            "#93c5fd",
            "第一種住居地域",
            "#a78bfa",
            "第二種住居地域",
            "#c4b5fd",
            "準住居地域",
            "#e9d5ff",
            "近隣商業地域",
            "#fbbf24",
            "商業地域",
            "#f97316",
            "準工業地域",
            "#a3e635",
            "工業地域",
            "#6b7280",
            "工業専用地域",
            "#374151",
            "#6b7280",
          ],
          "fill-opacity": 0.35,
        }}
      />
    </Source>
  );
}
