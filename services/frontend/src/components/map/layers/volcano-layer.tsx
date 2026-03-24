"use client";

import { Layer, Source } from "react-map-gl/maplibre";
import { useStaticLayer } from "@/hooks/use-static-layer";

interface Props {
  visible: boolean;
}

export function VolcanoLayer({ visible }: Props) {
  const { data } = useStaticLayer("national", "volcano", visible);
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
