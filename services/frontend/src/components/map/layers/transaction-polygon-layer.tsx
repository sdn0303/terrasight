"use client";

import type { GeoJSON } from "geojson";
import { Layer, Source } from "react-map-gl/mapbox";
import type { TransactionAggregation } from "@/lib/api/schemas/transaction-aggregation";
import { PRICE_STOPS, TX_COUNT_THRESHOLDS } from "@/lib/palette";

interface Props {
  data: TransactionAggregation;
  visible: boolean;
}

export function TransactionPolygonLayer({ data, visible }: Props) {
  if (!visible) return null;
  return (
    // coordinates: unknown in Zod schema; cast at react-map-gl boundary after API validation
    <Source
      id="transaction-aggregation"
      type="geojson"
      data={data as unknown as GeoJSON}
    >
      <Layer
        id="transaction_polygon"
        type="fill"
        paint={{
          "fill-color": [
            "interpolate",
            ["linear"],
            ["get", "tx_count"],
            TX_COUNT_THRESHOLDS.min,
            PRICE_STOPS.min,
            TX_COUNT_THRESHOLDS.low,
            PRICE_STOPS.low,
            TX_COUNT_THRESHOLDS.mid,
            PRICE_STOPS.mid,
            TX_COUNT_THRESHOLDS.midHigh,
            PRICE_STOPS.midHigh,
            TX_COUNT_THRESHOLDS.high,
            PRICE_STOPS.high,
            TX_COUNT_THRESHOLDS.max,
            PRICE_STOPS.max,
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
              { locale: "ja-JP", "min-fraction-digits": 0 },
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
