"use client";

import { Layer, Source } from "react-map-gl/maplibre";

interface Props {
  visible: boolean;
}

/**
 * Tsunami inundation risk layer (津波浸水想定区域).
 *
 * Color by expected inundation depth:
 * - < 1m → light blue
 * - 1-3m → blue
 * - 3-5m → dark blue
 * - > 5m → purple
 */
export function TsunamiLayer({ visible }: Props) {
  if (!visible) return null;
  return (
    <Source id="tsunami" type="geojson" data="/geojson/tsunami-tokyo.geojson">
      <Layer
        id="tsunami-fill"
        type="fill"
        paint={{
          "fill-color": [
            "interpolate",
            ["linear"],
            ["get", "depth"],
            0,
            "#bfdbfe",
            1,
            "#60a5fa",
            3,
            "#2563eb",
            5,
            "#7c3aed",
          ] as unknown as string,
          "fill-opacity": 0.45,
        }}
      />
      <Layer
        id="tsunami-outline"
        type="line"
        paint={{
          "line-color": "#38bdf8",
          "line-width": 1,
          "line-opacity": 0.6,
        }}
      />
    </Source>
  );
}
