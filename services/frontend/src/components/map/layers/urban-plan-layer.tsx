"use client";

import { Layer, Source } from "react-map-gl/maplibre";

interface Props {
  visible: boolean;
}

/**
 * Urban planning / location optimization zones (立地適正化区域).
 *
 * Color by zone type:
 * - 居住誘導区域 (residential) → emerald
 * - 都市機能誘導区域 (urban function) → teal
 */
export function UrbanPlanLayer({ visible }: Props) {
  if (!visible) return null;
  return (
    <Source
      id="urban-plan"
      type="geojson"
      data="/geojson/urban-plan-tokyo.geojson"
    >
      <Layer
        id="urban-plan-fill"
        type="fill"
        minzoom={11}
        paint={{
          "fill-color": [
            "match",
            ["get", "zoneType"],
            "居住誘導区域",
            "#34d399",
            "都市機能誘導区域",
            "#2dd4bf",
            "#6ee7b7",
          ] as unknown as string,
          "fill-opacity": 0.3,
        }}
      />
      <Layer
        id="urban-plan-outline"
        type="line"
        minzoom={11}
        paint={{
          "line-color": "#34d399",
          "line-width": 1.5,
          "line-opacity": 0.7,
        }}
      />
    </Source>
  );
}
