"use client";

import "maplibre-gl/dist/maplibre-gl.css";

import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";
import { Map, NavigationControl } from "react-map-gl/maplibre";
import type {
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

  if (!mounted) {
    return (
      <div
        className="flex items-center justify-center"
        style={{
          width: "100%",
          height: "100%",
          background: "var(--bg-primary)",
          color: "var(--accent-cyan)",
          fontFamily: "var(--font-mono)",
          fontSize: "14px",
        }}
      >
        LOADING MAP...
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
      mapStyle={MAP_CONFIG.style}
      style={{ width: "100%", height: "100%" }}
      attributionControl={false}
    >
      <NavigationControl position="bottom-right" />
      {children}
    </Map>
  );
}
