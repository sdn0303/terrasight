"use client";

import type { GeoJSON } from "geojson";
import { Layer, Source } from "react-map-gl/mapbox";
import type { LandPriceAggregation } from "@/lib/api/schemas/land-price-aggregation";
import { PRICE_STOPS, PRICE_THRESHOLDS } from "@/lib/palette";

interface Props {
  data: LandPriceAggregation;
  visible: boolean;
}

export function LandPricePolygonLayer({ data, visible }: Props) {
  if (!visible) return null;
  return (
    // coordinates: unknown in Zod schema; cast at react-map-gl boundary after API validation
    <Source
      id="land-price-aggregation"
      type="geojson"
      data={data as unknown as GeoJSON}
    >
      <Layer
        id="land_price_polygon"
        type="fill"
        paint={{
          "fill-color": [
            "interpolate",
            ["linear"],
            ["get", "avg_price"],
            PRICE_THRESHOLDS.min,
            PRICE_STOPS.min,
            PRICE_THRESHOLDS.low,
            PRICE_STOPS.low,
            PRICE_THRESHOLDS.mid,
            PRICE_STOPS.mid,
            PRICE_THRESHOLDS.midHigh,
            PRICE_STOPS.midHigh,
            PRICE_THRESHOLDS.high,
            PRICE_STOPS.high,
            PRICE_THRESHOLDS.max,
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
              { locale: "ja-JP", "min-fraction-digits": 0 },
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
