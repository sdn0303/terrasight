"use client";

import type { FeatureCollection } from "geojson";
import { Layer, Source } from "react-map-gl/mapbox";
import { PRICE_STOPS } from "@/lib/palette";

interface Props {
  data: FeatureCollection;
  visible: boolean;
}

export function LandPricePolygonLayer({ data, visible }: Props) {
  if (!visible) return null;
  return (
    <Source id="land-price-aggregation" type="geojson" data={data}>
      <Layer
        id="land_price_polygon"
        type="fill"
        paint={{
          "fill-color": [
            "interpolate",
            ["linear"],
            ["get", "avg_price"],
            50_000,
            PRICE_STOPS.min,
            200_000,
            PRICE_STOPS.low,
            500_000,
            PRICE_STOPS.mid,
            1_000_000,
            PRICE_STOPS.midHigh,
            2_000_000,
            PRICE_STOPS.high,
            5_000_000,
            PRICE_STOPS.max,
          ],
          "fill-opacity": 0.6,
          "fill-opacity-transition": { duration: 300, delay: 0 },
        }}
      />
      <Layer
        id="land_price_polygon_label"
        type="symbol"
        layout={{
          "text-field": [
            "concat",
            "¥",
            [
              "number-format",
              ["get", "avg_price"],
              { "min-fraction-digits": 0 },
            ],
          ],
          "text-size": 12,
        }}
        paint={{
          "text-color": "#1e293b",
          "text-halo-color": "#ffffff",
          "text-halo-width": 1,
        }}
      />
    </Source>
  );
}
