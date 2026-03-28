"use client";

import type { FeatureCollection } from "geojson";
import { Layer, Source } from "react-map-gl/maplibre";
import { useStaticLayer } from "@/hooks/use-static-layer";

interface Props {
  visible: boolean;
  data?: FeatureCollection;
}

export function DIDLayer({ visible, data: propData }: Props) {
  const selfFetch = useStaticLayer("13", "did", visible && !propData);
  const data = propData ?? selfFetch.data;
  if (!visible || !data) return null;
  return (
    <Source id="did" type="geojson" data={data}>
      <Layer
        id="did-fill"
        type="fill"
        paint={{
          "fill-color": [
            "interpolate",
            ["linear"],
            ["get", "population"],
            0,
            "#1a1a25",
            50000,
            "#164e63",
            100000,
            "#0e7490",
            200000,
            "#06b6d4",
            500000,
            "#22d3ee",
          ] as unknown as string,
          "fill-opacity": 0.4,
        }}
      />
      <Layer
        id="did-outline"
        type="line"
        paint={{
          "line-color": "#22d3ee",
          "line-width": 1,
          "line-opacity": 0.6,
        }}
      />
    </Source>
  );
}
