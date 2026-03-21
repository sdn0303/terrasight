"use client";

import "maplibre-gl/dist/maplibre-gl.css";

import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";
import { Map, NavigationControl } from "react-map-gl/maplibre";
import type {
  MapEvent,
  MapLayerMouseEvent,
  ViewStateChangeEvent,
} from "react-map-gl/maplibre";
import { DEBOUNCE_MS, MAP_CONFIG } from "@/lib/constants";
import { useMapStore } from "@/stores/map-store";

interface MapViewProps {
  children?: ReactNode;
  onMoveEnd?: () => void;
  onFeatureClick?: (e: MapLayerMouseEvent) => void;
}

export function MapView({ children, onMoveEnd, onFeatureClick }: MapViewProps) {
  const [mounted, setMounted] = useState(false);
  const { viewState, setViewState } = useMapStore();
  const moveEndTimerRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  useEffect(() => {
    setMounted(true);
  }, []);

  useEffect(() => {
    return () => {
      if (moveEndTimerRef.current) clearTimeout(moveEndTimerRef.current);
    };
  }, []);

  const handleMove = useCallback(
    (e: ViewStateChangeEvent) => {
      setViewState(e.viewState);
    },
    [setViewState],
  );

  const handleMoveEnd = useCallback(
    (_e: ViewStateChangeEvent) => {
      if (moveEndTimerRef.current) clearTimeout(moveEndTimerRef.current);
      moveEndTimerRef.current = setTimeout(() => {
        onMoveEnd?.();
      }, DEBOUNCE_MS);
    },
    [onMoveEnd],
  );

  const handleClick = useCallback(
    (e: MapLayerMouseEvent) => {
      onFeatureClick?.(e);
    },
    [onFeatureClick],
  );

  const handleLoad = useCallback((e: MapEvent) => {
    const map = e.target;

    // Add terrain DEM source for 3D elevation
    map.addSource("terrain-dem", {
      type: "raster-dem",
      tiles: [
        "https://s3.amazonaws.com/elevation-tiles-prod/terrainrgb/{z}/{x}/{y}.png",
      ],
      tileSize: 256,
      maxzoom: 15,
      encoding: "terrarium",
    });

    // Enable terrain with mild exaggeration to reveal urban topography
    map.setTerrain({ source: "terrain-dem", exaggeration: 1.5 });

    // Add 3D building extrusion layer using CARTO vector tiles
    // CARTO Dark Matter exposes a 'building' layer in the 'carto' source
    const style = map.getStyle();
    const layers = style.layers ?? [];
    const hasBuildingLayer = layers.some(
      (l) => l.id === "building" || ("source-layer" in l && l["source-layer"] === "building"),
    );

    if (!hasBuildingLayer) {
      // Find insertion point: above roads, below labels
      const labelLayerId = layers.find(
        (l) =>
          l.type === "symbol" &&
          "source-layer" in l &&
          typeof l["source-layer"] === "string" &&
          l["source-layer"].includes("place"),
      )?.id;

      map.addLayer(
        {
          id: "3d-buildings",
          type: "fill-extrusion",
          source: "carto",
          "source-layer": "building",
          filter: ["==", ["geometry-type"], "Polygon"],
          paint: {
            "fill-extrusion-color": "#1e1e2e",
            "fill-extrusion-height": [
              "interpolate",
              ["linear"],
              ["zoom"],
              14,
              0,
              16,
              ["coalesce", ["get", "render_height"], ["get", "height"], 10],
            ],
            "fill-extrusion-base": [
              "interpolate",
              ["linear"],
              ["zoom"],
              14,
              0,
              16,
              ["coalesce", ["get", "render_min_height"], ["get", "min_height"], 0],
            ],
            "fill-extrusion-opacity": 0.7,
          },
        },
        labelLayerId,
      );
    }
  }, []);

  if (!mounted) {
    return (
      <div
        className="flex items-center justify-center"
        style={{
          width: "100%",
          height: "100%",
          background: "var(--bg-primary)",
          color: "var(--text-secondary)",
          fontFamily: "var(--font-mono)",
          fontSize: "13px",
          letterSpacing: "0.1em",
        }}
      >
        地層 LOADING...
      </div>
    );
  }

  return (
    <Map
      longitude={viewState.longitude}
      latitude={viewState.latitude}
      zoom={viewState.zoom}
      pitch={viewState.pitch}
      bearing={viewState.bearing}
      onMove={handleMove}
      onMoveEnd={handleMoveEnd}
      onClick={handleClick}
      onLoad={handleLoad}
      mapStyle={MAP_CONFIG.style}
      style={{ width: "100%", height: "100%" }}
      attributionControl={false}
    >
      <NavigationControl position="bottom-right" />
      {children}
    </Map>
  );
}
