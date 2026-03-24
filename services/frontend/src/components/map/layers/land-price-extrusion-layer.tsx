"use client";

import type { FeatureCollection } from "geojson";
import { Layer, Source } from "react-map-gl/maplibre";
import {
  PRICE_COLOR_STOPS,
  PRICE_HEIGHT_STOPS,
} from "@/features/land-prices/constants";
import { useMediaQuery } from "@/hooks/use-media-query";

interface Props {
  data: FeatureCollection;
  visible: boolean;
  isFetching?: boolean;
}

export function LandPriceExtrusionLayer({
  data,
  visible,
  isFetching = false,
}: Props) {
  const isMobile = useMediaQuery("(max-width: 767px)");

  if (!visible || data.features.length === 0) return null;

  // Decision 9: mobile renders only circle layer at all zoom levels
  if (isMobile) {
    return (
      <Source id="land-price-extrusion" type="geojson" data={data}>
        <Layer
          id="land-price-ts-circle"
          type="circle"
          maxzoom={16}
          paint={{
            "circle-radius": ["interpolate", ["linear"], ["zoom"], 8, 2, 11, 5],
            "circle-color": [
              "interpolate",
              ["linear"],
              ["get", "price_per_sqm"],
              ...PRICE_COLOR_STOPS,
            ],
            "circle-opacity": isFetching ? 0.3 : 0.85,
            "circle-stroke-width": 1,
            "circle-stroke-color": "#0c0c14",
          }}
        />
      </Source>
    );
  }

  // Decision 2: opacity target values — fades to 0.3 while fetching
  const extrusionOpacityTarget = isFetching ? 0.3 : 0.8;
  const circleOpacityTarget = isFetching ? 0.3 : 0.85;

  return (
    <Source id="land-price-extrusion" type="geojson" data={data}>
      {/* Decision 5: 3D columns begin fading in at zoom 11, fully opaque at 12 */}
      <Layer
        id="land-price-extrusion-3d"
        type="fill-extrusion"
        minzoom={11}
        paint={{
          "fill-extrusion-color": [
            "interpolate",
            ["linear"],
            ["get", "price_per_sqm"],
            ...PRICE_COLOR_STOPS,
          ],
          "fill-extrusion-height": [
            "interpolate",
            ["linear"],
            ["get", "price_per_sqm"],
            ...PRICE_HEIGHT_STOPS,
          ],
          "fill-extrusion-base": 0,
          "fill-extrusion-opacity": [
            "interpolate",
            ["linear"],
            ["zoom"],
            11,
            0,
            12,
            extrusionOpacityTarget,
          ],
        }}
      />

      {/* Decision 5: 2D circles fade out across zoom 12→13 for smooth LOD crossfade */}
      <Layer
        id="land-price-ts-circle"
        type="circle"
        maxzoom={13}
        paint={{
          "circle-radius": ["interpolate", ["linear"], ["zoom"], 8, 2, 11, 5],
          "circle-color": [
            "interpolate",
            ["linear"],
            ["get", "price_per_sqm"],
            ...PRICE_COLOR_STOPS,
          ],
          "circle-opacity": [
            "interpolate",
            ["linear"],
            ["zoom"],
            12,
            circleOpacityTarget,
            13,
            0,
          ],
          "circle-stroke-width": 1,
          "circle-stroke-color": "#0c0c14",
        }}
      />
    </Source>
  );
}
