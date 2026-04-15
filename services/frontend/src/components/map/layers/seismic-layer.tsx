"use client";

import type { FeatureCollection } from "geojson";
import type { FilterSpecification } from "mapbox-gl";
import { Layer, Source } from "react-map-gl/mapbox";
import { useStaticLayer } from "@/hooks/use-static-layer";

interface Props {
  visible: boolean;
  data?: FeatureCollection;
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
export function SeismicLayer({ visible, data: propData }: Props) {
  const selfFetch = useStaticLayer("seismic", visible && !propData);
  const data = propData ?? selfFetch.data;
  if (!visible || !data) return null;
  return (
    <Source id="seismic" type="geojson" data={data}>
      {/* Polygon fill for fault rupture zones */}
      <Layer
        id="seismic-fill"
        type="fill"
        filter={["==", ["geometry-type"], "Polygon"] as FilterSpecification}
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
        filter={["==", ["geometry-type"], "Polygon"] as FilterSpecification}
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
        filter={["==", ["geometry-type"], "LineString"] as FilterSpecification}
        paint={{
          "line-color": "#fbbf24",
          "line-width": 2,
          "line-opacity": 0.8,
        }}
      />
    </Source>
  );
}
