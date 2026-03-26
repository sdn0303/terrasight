"use client";

import { Layer, Source } from "react-map-gl/maplibre";

export function BoundaryLayer() {
  return (
    <>
      <Source id="n03-pref" type="geojson" data="/geojson/n03/prefectures.geojson">
        <Layer
          id="n03-pref-line"
          type="line"
          paint={{
            "line-color": "#ffffff",
            "line-width": ["interpolate", ["linear"], ["zoom"], 4, 0.8, 10, 2],
            "line-opacity": ["interpolate", ["linear"], ["zoom"], 4, 0.6, 10, 0.8],
          }}
        />
        <Layer
          id="n03-pref-label"
          type="symbol"
          layout={{
            "text-field": ["get", "name"],
            "text-size": ["interpolate", ["linear"], ["zoom"], 4, 10, 8, 14],
            "text-anchor": "center",
          }}
          paint={{
            "text-color": "#ffffff",
            "text-opacity": ["interpolate", ["linear"], ["zoom"], 4, 0.7, 11, 0],
            "text-halo-color": "#000000",
            "text-halo-width": 1,
          }}
        />
      </Source>

      <Source id="n03-muni" type="geojson" data="/geojson/n03/municipalities.geojson">
        <Layer
          id="n03-muni-line"
          type="line"
          paint={{
            "line-color": "#ffffff",
            "line-width": 0.5,
            "line-opacity": ["interpolate", ["linear"], ["zoom"], 7, 0, 8, 0.3, 12, 0.6],
          }}
        />
        <Layer
          id="n03-muni-label"
          type="symbol"
          minzoom={8}
          layout={{
            "text-field": ["get", "name"],
            "text-size": ["interpolate", ["linear"], ["zoom"], 8, 0, 10, 10, 14, 13],
            "text-anchor": "center",
          }}
          paint={{
            "text-color": "#ffffff",
            "text-opacity": ["interpolate", ["linear"], ["zoom"], 8, 0, 10, 0.5, 14, 0.8],
            "text-halo-color": "#000000",
            "text-halo-width": 1,
          }}
        />
      </Source>
    </>
  );
}
