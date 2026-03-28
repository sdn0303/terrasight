"use client";

import type { FeatureCollection } from "geojson";
import { Layer, Source } from "react-map-gl/maplibre";
import { useStaticLayer } from "@/hooks/use-static-layer";

interface Props {
  visible: boolean;
  data?: FeatureCollection;
}

export function VolcanoLayer({ visible, data: propData }: Props) {
  const selfFetch = useStaticLayer("national", "volcano", visible && !propData);
  const data = propData ?? selfFetch.data;
  if (!visible || !data) return null;
  return (
    <Source id="volcano" type="geojson" data={data}>
      <Layer
        id="volcano-circle"
        type="circle"
        paint={{
          "circle-radius": 8,
          "circle-color": "#ef4444",
          "circle-stroke-width": 2,
          "circle-stroke-color": "#fbbf24",
          "circle-opacity": 0.9,
        }}
      />
    </Source>
  );
}
