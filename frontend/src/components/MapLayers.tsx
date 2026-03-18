"use client";

import { Source, Layer } from "react-map-gl/maplibre";
import type maplibregl from "maplibre-gl";
import type { ActiveLayers } from "@/lib/layers";

interface MapLayersProps {
  layerData: Record<string, GeoJSON.FeatureCollection>;
  activeLayers: ActiveLayers;
}

// 用途地域の色マッピング（プロパティ値→色）
// The MLIT API returns a "youto" or similar property with zone type codes
// We use a match expression to color-code them
const ZONE_FILL_COLOR = [
  "match",
  ["get", "youto"],
  // 住居系 (blue shades)
  "第一種低層住居専用地域", "#2563eb",
  "第二種低層住居専用地域", "#3b82f6",
  "第一種中高層住居専用地域", "#60a5fa",
  "第二種中高層住居専用地域", "#93c5fd",
  "第一種住居地域", "#a78bfa",
  "第二種住居地域", "#c4b5fd",
  "準住居地域", "#e9d5ff",
  // 商業系 (warm tones)
  "近隣商業地域", "#fbbf24",
  "商業地域", "#f97316",
  // 工業系 (gray/green)
  "準工業地域", "#a3e635",
  "工業地域", "#6b7280",
  "工業専用地域", "#374151",
  // default
  "#52525b",
] as unknown as maplibregl.ExpressionSpecification;

export default function MapLayers({ layerData, activeLayers }: MapLayersProps) {
  return (
    <>
      {/* ===== 用途地域ポリゴン (Zoning) ===== */}
      {activeLayers.zoning && layerData.zoning && (
        <Source id="zoning-source" type="geojson" data={layerData.zoning}>
          <Layer
            id="zoning-fill"
            type="fill"
            paint={{
              "fill-color": ZONE_FILL_COLOR,
              "fill-opacity": 0.35,
            }}
          />
          <Layer
            id="zoning-outline"
            type="line"
            paint={{
              "line-color": "rgba(255,255,255,0.15)",
              "line-width": 0.5,
            }}
          />
        </Source>
      )}

      {/* ===== 液状化リスク 3D (Liquefaction) ===== */}
      {activeLayers.liquefaction && layerData.liquefaction && (
        <Source id="liquefaction-source" type="geojson" data={layerData.liquefaction}>
          <Layer
            id="liquefaction-3d"
            type="fill-extrusion"
            paint={{
              "fill-extrusion-color": [
                "interpolate",
                ["linear"],
                ["coalesce", ["get", "liquefaction_risk"], ["get", "risk_level"], 0.5],
                0, "#1a6fff",
                0.5, "#ffd000",
                1.0, "#e04030",
              ] as unknown as maplibregl.ExpressionSpecification,
              "fill-extrusion-height": [
                "*",
                ["coalesce", ["get", "liquefaction_risk"], ["get", "risk_level"], 0.5],
                200,
              ] as unknown as maplibregl.ExpressionSpecification,
              "fill-extrusion-base": 0,
              "fill-extrusion-opacity": 0.7,
            }}
          />
        </Source>
      )}

      {/* ===== 洪水浸水 3D (Flood) ===== */}
      {activeLayers.flood && layerData.flood && (
        <Source id="flood-source" type="geojson" data={layerData.flood}>
          <Layer
            id="flood-3d"
            type="fill-extrusion"
            paint={{
              "fill-extrusion-color": [
                "interpolate",
                ["linear"],
                ["coalesce", ["get", "depth"], ["get", "flood_depth"], 1],
                0, "#1a6fff",
                2, "#ffd000",
                5, "#e04030",
              ] as unknown as maplibregl.ExpressionSpecification,
              "fill-extrusion-height": [
                "*",
                ["coalesce", ["get", "depth"], ["get", "flood_depth"], 1],
                40,
              ] as unknown as maplibregl.ExpressionSpecification,
              "fill-extrusion-base": 0,
              "fill-extrusion-opacity": 0.6,
            }}
          />
        </Source>
      )}

      {/* ===== 急傾斜地 (Steep Slope) ===== */}
      {activeLayers.steep_slope && layerData.steep_slope && (
        <Source id="steep-slope-source" type="geojson" data={layerData.steep_slope}>
          <Layer
            id="steep-slope-fill"
            type="fill"
            paint={{
              "fill-color": "#e04030",
              "fill-opacity": 0.4,
            }}
          />
          <Layer
            id="steep-slope-outline"
            type="line"
            paint={{
              "line-color": "#e04030",
              "line-width": 1.5,
              "line-dasharray": [2, 2],
            }}
          />
        </Source>
      )}

      {/* ===== 地価公示ポイント (Land Prices) ===== */}
      {activeLayers.landprice && layerData.landprice && (
        <Source id="landprice-source" type="geojson" data={layerData.landprice}>
          <Layer
            id="landprice-circle"
            type="circle"
            paint={{
              "circle-radius": [
                "interpolate", ["linear"], ["zoom"],
                10, 3,
                15, 8,
              ],
              "circle-color": "#22d3ee",
              "circle-opacity": 0.8,
              "circle-stroke-width": 1,
              "circle-stroke-color": "#0a0a0f",
            }}
          />
        </Source>
      )}

      {/* ===== 学校 (Schools) ===== */}
      {activeLayers.schools && layerData.schools && (
        <Source id="schools-source" type="geojson" data={layerData.schools}>
          <Layer
            id="schools-circle"
            type="circle"
            paint={{
              "circle-radius": [
                "interpolate", ["linear"], ["zoom"],
                10, 2,
                15, 6,
              ],
              "circle-color": "#10b981",
              "circle-opacity": 0.9,
              "circle-stroke-width": 1.5,
              "circle-stroke-color": "#065f46",
            }}
          />
        </Source>
      )}

      {/* ===== 医療機関 (Medical) ===== */}
      {activeLayers.medical && layerData.medical && (
        <Source id="medical-source" type="geojson" data={layerData.medical}>
          <Layer
            id="medical-circle"
            type="circle"
            paint={{
              "circle-radius": [
                "interpolate", ["linear"], ["zoom"],
                10, 2,
                15, 6,
              ],
              "circle-color": "#f472b6",
              "circle-opacity": 0.9,
              "circle-stroke-width": 1.5,
              "circle-stroke-color": "#9d174d",
            }}
          />
        </Source>
      )}
    </>
  );
}
