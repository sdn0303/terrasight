"use client";

import { Layer, Source } from "react-map-gl/maplibre";

interface Props {
  visible: boolean;
}

/**
 * Landslide risk layer (土砂災害警戒区域).
 *
 * Color mapping by designation type:
 * - 土砂災害警戒区域 (yellow zone) → warning orange
 * - 土砂災害特別警戒区域 (red zone) → danger red
 */
export function LandslideLayer({ visible }: Props) {
  if (!visible) return null;
  return (
    <Source
      id="landslide"
      type="geojson"
      data="/geojson/landslide-tokyo.geojson"
    >
      <Layer
        id="landslide-fill"
        type="fill"
        minzoom={11}
        paint={{
          "fill-color": [
            "match",
            ["get", "designation"],
            "特別警戒区域",
            "#ef4444",
            "#fb923c",
          ] as unknown as string,
          "fill-opacity": 0.4,
        }}
      />
      <Layer
        id="landslide-outline"
        type="line"
        minzoom={11}
        paint={{
          "line-color": "#fb923c",
          "line-width": 1,
          "line-opacity": 0.6,
        }}
      />
    </Source>
  );
}
