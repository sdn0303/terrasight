"use client";

import { useCallback, type ReactNode } from "react";
import Map, { NavigationControl, type ViewStateChangeEvent } from "react-map-gl/maplibre";
import "maplibre-gl/dist/maplibre-gl.css";

const CARTO_DARK_MATTER = "https://basemaps.cartocdn.com/gl/dark-matter-gl-style/style.json";

interface MapViewProps {
  children?: ReactNode;
  onMoveEnd?: (bounds: { north: number; south: number; east: number; west: number }) => void;
}

export default function MapView({ children, onMoveEnd }: MapViewProps) {
  const handleMoveEnd = useCallback(
    (evt: ViewStateChangeEvent) => {
      if (!onMoveEnd) return;
      const map = evt.target;
      const bounds = map.getBounds();
      onMoveEnd({
        north: bounds.getNorth(),
        south: bounds.getSouth(),
        east: bounds.getEast(),
        west: bounds.getWest(),
      });
    },
    [onMoveEnd],
  );

  return (
    <Map
      initialViewState={{
        longitude: 139.767,
        latitude: 35.681,
        zoom: 12,
        pitch: 45,
      }}
      style={{ width: "100%", height: "100%" }}
      mapStyle={CARTO_DARK_MATTER}
      onMoveEnd={handleMoveEnd}
    >
      <NavigationControl position="bottom-right" />
      {children}
    </Map>
  );
}
