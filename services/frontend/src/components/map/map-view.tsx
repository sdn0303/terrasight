"use client";

import "mapbox-gl/dist/mapbox-gl.css";

import type { Map as MapboxMap } from "mapbox-gl";
import {
  type ReactNode,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import type {
  MapEvent,
  MapMouseEvent,
  ViewStateChangeEvent,
} from "react-map-gl/mapbox";
import { Map as MapGL, NavigationControl } from "react-map-gl/mapbox";
import { useMapUrlState } from "@/hooks/use-map-url-state";
import type { BBox } from "@/lib/api";
import { DEBOUNCE_MS } from "@/lib/constants";
import { ALL_INTERACTIVE_LAYER_IDS } from "@/lib/layers";
import { logger } from "@/lib/logger";
import { useMapStore } from "@/stores/map-store";
import type { BaseMap } from "@/stores/ui-store";
import { useUIStore } from "@/stores/ui-store";

const MAPBOX_STYLES = {
  light: "mapbox://styles/mapbox/streets-v12",
  dark: "mapbox://styles/mapbox/dark-v11",
  satellite: "mapbox://styles/mapbox/satellite-streets-v12",
} as const satisfies Record<BaseMap, string>;

const log = logger.child({ module: "map-view" });

interface MapViewProps {
  children?: ReactNode;
  onMoveEnd?: (bbox: BBox) => void;
  onFeatureClick?: (e: MapMouseEvent) => void;
}

const WEBGL_RECOVERY_TIMEOUT_MS = 5000;

export function MapView({ children, onMoveEnd, onFeatureClick }: MapViewProps) {
  useMapUrlState();
  const [mounted, setMounted] = useState(false);
  const [webglLost, setWebglLost] = useState(false);
  const { viewState, setViewState } = useMapStore();
  const baseMap = useUIStore((s) => s.baseMap);
  const moveEndTimerRef = useRef<ReturnType<typeof setTimeout> | undefined>(
    undefined,
  );
  const mapRef = useRef<MapboxMap | null>(null);

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
        const map = mapRef.current;
        if (!map || !onMoveEnd) return;
        const bounds = map.getBounds();
        if (!bounds) return;
        onMoveEnd({
          south: bounds.getSouth(),
          west: bounds.getWest(),
          north: bounds.getNorth(),
          east: bounds.getEast(),
        });
      }, DEBOUNCE_MS);
    },
    [onMoveEnd],
  );

  const handleClick = useCallback(
    (e: MapMouseEvent) => {
      onFeatureClick?.(e);
    },
    [onFeatureClick],
  );

  const handleLoad = useCallback(
    (e: MapEvent) => {
      const map = e.target;
      mapRef.current = map;

      // Emit initial real bbox so queries don't rely on the center+zoom approximation
      const bounds = map.getBounds();
      if (bounds) {
        onMoveEnd?.({
          south: bounds.getSouth(),
          west: bounds.getWest(),
          north: bounds.getNorth(),
          east: bounds.getEast(),
        });
      }

      // ─── CRITICAL GAP FIX: try/catch wrapper for addSource ───
      // Protects against malformed data or duplicate source IDs
      try {
        map.addSource("terrain-dem", {
          type: "raster-dem",
          url: "mapbox://mapbox.mapbox-terrain-dem-v1",
          tileSize: 512,
          maxzoom: 14,
        });

        map.setTerrain({ source: "terrain-dem", exaggeration: 1.5 });
      } catch (err) {
        log.error({ err }, "failed to add terrain source");
      }

      // Add 3D building extrusion layer using CARTO vector tiles
      try {
        const style = map.getStyle();
        const layers = style.layers ?? [];
        const hasBuildingLayer = layers.some(
          (l) =>
            l.id === "building" ||
            ("source-layer" in l && l["source-layer"] === "building"),
        );

        if (!hasBuildingLayer) {
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
                  [
                    "coalesce",
                    ["get", "render_min_height"],
                    ["get", "min_height"],
                    0,
                  ],
                ],
                "fill-extrusion-opacity": 0.7,
              },
            },
            labelLayerId,
          );
        }
      } catch (err) {
        log.error({ err }, "failed to add 3D buildings layer");
      }

      // ─── WebGL context lost recovery ───
      const canvas = map.getCanvas();
      const handleContextLost = (event: Event) => {
        event.preventDefault();
        log.warn("webgl context lost — attempting recovery");
        setWebglLost(true);
      };
      const handleContextRestored = () => {
        log.info("webgl context restored");
        setWebglLost(false);
      };

      canvas.addEventListener("webglcontextlost", handleContextLost);
      canvas.addEventListener("webglcontextrestored", handleContextRestored);

      // Cleanup: remove listeners when map is unmounted
      return () => {
        canvas.removeEventListener("webglcontextlost", handleContextLost);
        canvas.removeEventListener(
          "webglcontextrestored",
          handleContextRestored,
        );
      };
    },
    [onMoveEnd],
  );

  const handleForceReload = useCallback(() => {
    if (mapRef.current) {
      try {
        mapRef.current.triggerRepaint();
      } catch {
        // If triggerRepaint fails, force page-level recovery
        window.location.reload();
      }
    }
  }, []);

  // Auto-recovery timeout for WebGL context lost
  useEffect(() => {
    if (!webglLost) return;

    const timer = setTimeout(() => {
      // If still lost after timeout, attempt triggerRepaint as last resort
      if (mapRef.current) {
        try {
          mapRef.current.triggerRepaint();
          log.info("attempted triggerRepaint after WebGL timeout");
        } catch {
          log.warn(
            "triggerRepaint failed after WebGL timeout — user must reload",
          );
        }
      }
    }, WEBGL_RECOVERY_TIMEOUT_MS);

    return () => clearTimeout(timer);
  }, [webglLost]);

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
    <>
      <MapGL
        longitude={viewState.longitude}
        latitude={viewState.latitude}
        zoom={viewState.zoom}
        pitch={viewState.pitch}
        bearing={viewState.bearing}
        onMove={handleMove}
        onMoveEnd={handleMoveEnd}
        onClick={handleClick}
        onLoad={handleLoad}
        mapStyle={MAPBOX_STYLES[baseMap]}
        mapboxAccessToken={import.meta.env.VITE_MAPBOX_TOKEN}
        style={{ width: "100%", height: "100%" }}
        attributionControl={false}
        interactiveLayerIds={ALL_INTERACTIVE_LAYER_IDS}
      >
        <NavigationControl position="bottom-right" />
        {children}
      </MapGL>

      {/* WebGL context lost toast overlay */}
      {webglLost && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center"
          style={{ background: "rgba(12, 12, 20, 0.85)" }}
          role="alert"
          aria-live="assertive"
        >
          <div
            className="rounded-lg px-6 py-4 text-center max-w-xs"
            style={{
              background: "var(--bg-secondary)",
              border: "1px solid var(--border-primary)",
            }}
          >
            <div
              className="text-sm mb-2"
              style={{
                color: "var(--accent-warning)",
                fontFamily: "var(--font-mono)",
              }}
            >
              ⚠ 地図を再読み込み中...
            </div>
            <div
              className="text-xs mb-3"
              style={{ color: "var(--text-secondary)" }}
            >
              GPUメモリ不足が発生しました
            </div>
            <button
              type="button"
              onClick={handleForceReload}
              className="px-4 py-1.5 rounded text-xs"
              style={{
                background: "var(--accent-primary)",
                color: "var(--bg-primary)",
                fontFamily: "var(--font-mono)",
              }}
            >
              再読み込み
            </button>
          </div>
        </div>
      )}
    </>
  );
}
