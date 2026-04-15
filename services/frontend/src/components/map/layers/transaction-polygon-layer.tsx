"use client";

import type { FeatureCollection } from "geojson";
import { Layer, Source } from "react-map-gl/mapbox";

/** Transaction count color ramp: few (blue) → many (red) */
const TX_COUNT_STOPS = {
  min: "#3b82f6",
  low: "#06b6d4",
  mid: "#10b981",
  midHigh: "#eab308",
  high: "#f97316",
  max: "#ef4444",
} as const;

interface Props {
  data: FeatureCollection;
  visible: boolean;
}

export function TransactionPolygonLayer({ data, visible }: Props) {
  if (!visible) return null;
  return (
    <Source id="transaction-aggregation" type="geojson" data={data}>
      <Layer
        id="transaction_polygon"
        type="fill"
        paint={{
          "fill-color": [
            "interpolate",
            ["linear"],
            ["get", "tx_count"],
            10,
            TX_COUNT_STOPS.min,
            50,
            TX_COUNT_STOPS.low,
            100,
            TX_COUNT_STOPS.mid,
            300,
            TX_COUNT_STOPS.midHigh,
            500,
            TX_COUNT_STOPS.high,
            1000,
            TX_COUNT_STOPS.max,
          ],
          "fill-opacity": 0.6,
          "fill-opacity-transition": { duration: 300, delay: 0 },
        }}
      />
      <Layer
        id="transaction_polygon_label"
        type="symbol"
        layout={{
          "text-field": [
            "concat",
            ["to-string", ["get", "tx_count"]],
            "件 ¥",
            [
              "number-format",
              ["get", "avg_price_sqm"],
              { "min-fraction-digits": 0 },
            ],
          ],
          "text-size": 11,
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
