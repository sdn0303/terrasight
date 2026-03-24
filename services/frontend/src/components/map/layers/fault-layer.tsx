"use client";

import { Layer, Source } from "react-map-gl/maplibre";
import { useStaticLayer } from "@/hooks/use-static-layer";

interface Props {
  visible: boolean;
}

export function FaultLayer({ visible }: Props) {
  const { data } = useStaticLayer("national", "fault", visible);
  if (!visible || !data) return null;
  return (
    <Source id="fault" type="geojson" data={data}>
      <Layer
        id="fault-confirmed"
        type="line"
        filter={["==", ["get", "faultType2"], "実在断層"] as unknown as boolean}
        paint={{
          "line-color": "#ef4444",
          "line-width": 2,
        }}
      />
      <Layer
        id="fault-inferred"
        type="line"
        filter={["!=", ["get", "faultType2"], "実在断層"] as unknown as boolean}
        paint={{
          "line-color": "#fbbf24",
          "line-width": 2,
          "line-dasharray": [4, 2],
        }}
      />
    </Source>
  );
}
