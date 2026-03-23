"use client";

import { Layer, Source } from "react-map-gl/maplibre";

interface Props {
  visible: boolean;
}

/**
 * Seismic hazard source zone layer (地震動・断層震源域).
 *
 * Source: JSHIS 震源断層モデル (NIED J-SHIS)
 * Geometry: Polygon (fault rupture zones) and LineString (surface traces)
 *
 * Color mapping by 30-year exceedance probability (AVR_T30P):
 * - < 0.001  → very low:    blue-grey
 * - 0.001–0.005 → low:      cyan
 * - 0.005–0.02  → moderate: yellow
 * - ≥ 0.02      → high:     red
 */
export function SeismicLayer({ visible }: Props) {
  if (!visible) return null;
  return (
    <Source
      id="seismic"
      type="geojson"
      data="/geojson/jshis-seismic-tokyo.geojson"
    >
      {/* Polygon fill for fault rupture zones */}
      <Layer
        id="seismic-fill"
        type="fill"
        filter={["==", ["geometry-type"], "Polygon"] as unknown as boolean}
        paint={{
          "fill-color": [
            "step",
            ["get", "AVR_T30P"],
            "#52525b",
            0.001,
            "#06b6d4",
            0.005,
            "#eab308",
            0.02,
            "#ef4444",
          ] as unknown as string,
          "fill-opacity": 0.35,
        }}
      />
      <Layer
        id="seismic-outline"
        type="line"
        filter={["==", ["geometry-type"], "Polygon"] as unknown as boolean}
        paint={{
          "line-color": "#ef4444",
          "line-width": 1,
          "line-opacity": 0.5,
        }}
      />
      {/* Line layer for surface fault traces */}
      <Layer
        id="seismic-trace"
        type="line"
        filter={["==", ["geometry-type"], "LineString"] as unknown as boolean}
        paint={{
          "line-color": "#fbbf24",
          "line-width": 2,
          "line-opacity": 0.8,
        }}
      />
    </Source>
  );
}
