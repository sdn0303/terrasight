"use client";

import {
  parseAsFloat,
  parseAsString,
  useQueryStates,
} from "nuqs";
import { useEffect, useRef } from "react";
import { useMapStore } from "@/stores/map-store";
import { MAP_CONFIG } from "@/lib/constants";

const mapParams = {
  lat: parseAsFloat.withDefault(MAP_CONFIG.center[1]),
  lng: parseAsFloat.withDefault(MAP_CONFIG.center[0]),
  z: parseAsFloat.withDefault(MAP_CONFIG.zoom),
  pitch: parseAsFloat.withDefault(MAP_CONFIG.pitch),
  bearing: parseAsFloat.withDefault(MAP_CONFIG.bearing),
  layers: parseAsString.withDefault("landprice,zoning"),
};

export function useMapUrlState() {
  const [params, setParams] = useQueryStates(mapParams, {
    history: "replace",
    shallow: true,
  });
  const initialized = useRef(false);
  const { viewState, setViewState, visibleLayers, toggleLayer } = useMapStore();

  // On mount: restore map state from URL
  useEffect(() => {
    if (initialized.current) return;
    initialized.current = true;

    setViewState({
      latitude: params.lat,
      longitude: params.lng,
      zoom: params.z,
      pitch: params.pitch,
      bearing: params.bearing,
    });

    // Sync layers from URL
    const urlLayers = new Set(params.layers.split(",").filter(Boolean));
    const currentLayers = useMapStore.getState().visibleLayers;
    for (const id of currentLayers) {
      if (!urlLayers.has(id)) toggleLayer(id);
    }
    for (const id of urlLayers) {
      if (!currentLayers.has(id)) toggleLayer(id);
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Sync store → URL on view state change
  useEffect(() => {
    if (!initialized.current) return;
    setParams({
      lat: Math.round(viewState.latitude * 10000) / 10000,
      lng: Math.round(viewState.longitude * 10000) / 10000,
      z: Math.round(viewState.zoom * 10) / 10,
      pitch: Math.round(viewState.pitch),
      bearing: Math.round(viewState.bearing),
      layers: [...visibleLayers].sort().join(","),
    });
  }, [viewState, visibleLayers, setParams]);
}
