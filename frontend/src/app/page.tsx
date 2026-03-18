"use client";

import dynamic from "next/dynamic";
import { useCallback, useEffect, useState } from "react";
import { fetchAPI } from "@/lib/api";
import { getDefaultActiveLayers, type ActiveLayers } from "@/lib/layers";
import { useMapData } from "@/hooks/useMapData";
import LayerPanel from "@/components/LayerPanel";
import MapLayers from "@/components/MapLayers";
import ScoreCard, { type ScoreCardData } from "@/components/ScoreCard";
import CRTOverlay from "@/components/CRTOverlay";

const MapView = dynamic(() => import("@/components/MapView"), { ssr: false });

type HealthStatus = "loading" | "online" | "offline";

export default function Home() {
  const [status, setStatus] = useState<HealthStatus>("loading");
  const [activeLayers, setActiveLayers] = useState<ActiveLayers>(getDefaultActiveLayers);
  const [scoreCardData, setScoreCardData] = useState<ScoreCardData | null>(null);
  const { layerData, loading, fetchForBounds } = useMapData(activeLayers);

  const handleMapClick = useCallback((features: GeoJSON.Feature[]) => {
    if (features.length === 0) {
      setScoreCardData(null);
      return;
    }
    const f = features[0];
    const props = f.properties || {};
    setScoreCardData({
      address: props.address || props.name || undefined,
      pricePerSqm: props.price_per_sqm || props.L01_006 || undefined,
      landPrice: props.land_price || props.L01_006 || undefined,
      zoneType: props.youto || props.zone_type || undefined,
      liquefactionRisk: props.liquefaction_risk || props.risk_level || undefined,
      floodDepth: props.depth || props.flood_depth || undefined,
      nearestSchool: props.school_name || props.P29_004 || undefined,
      nearestMedical: props.medical_name || props.P04_002 || undefined,
      properties: props,
    });
  }, []);

  useEffect(() => {
    fetchAPI<{ status: string }>("/api/health")
      .then(() => setStatus("online"))
      .catch(() => setStatus("offline"));
  }, []);

  return (
    <div className="relative w-screen h-screen">
      <MapView onMoveEnd={fetchForBounds} onClick={handleMapClick}>
        <MapLayers layerData={layerData} activeLayers={activeLayers} />
      </MapView>

      {/* Header overlay */}
      <div className="absolute top-6 left-[22rem] z-10 pointer-events-none">
        <h1
          className="text-xl font-bold tracking-[0.15em] leading-tight m-0"
          style={{ color: "var(--accent-cyan)" }}
        >
          不動産投資 VISUALIZER
        </h1>
        <p
          className="text-[0.65rem] tracking-[0.25em] mt-1 m-0"
          style={{ color: "var(--text-muted)" }}
        >
          MLIT GEOSPATIAL DATA PLATFORM
        </p>
      </div>

      {/* Layer panel */}
      <LayerPanel activeLayers={activeLayers} setActiveLayers={setActiveLayers} />

      {/* Loading indicator */}
      {loading && (
        <div
          className="absolute top-6 right-6 z-[300] flex items-center gap-2 px-4 py-2 rounded-md border text-[0.7rem] tracking-wider"
          style={{
            background: "var(--bg-secondary)",
            borderColor: "var(--border-primary)",
            color: "var(--accent-cyan)",
          }}
        >
          <span className="inline-block w-3 h-3 rounded-full border-2 border-current border-t-transparent animate-spin" />
          LOADING...
        </div>
      )}

      {/* Status indicator */}
      <div
        className="absolute bottom-4 left-1/2 -translate-x-1/2 z-10 flex items-center gap-2 px-4 py-1.5 rounded-md border text-[0.7rem] tracking-wider"
        style={{
          background: "var(--bg-secondary)",
          borderColor: "var(--border-primary)",
          color: "var(--text-secondary)",
        }}
      >
        <span
          className="w-1.5 h-1.5 rounded-full"
          style={{
            background:
              status === "online"
                ? "var(--accent-cyan)"
                : status === "offline"
                  ? "var(--accent-danger)"
                  : "var(--text-muted)",
          }}
        />
        BACKEND {status === "loading" ? "..." : status.toUpperCase()}
      </div>

      <ScoreCard data={scoreCardData} />
      <CRTOverlay />
    </div>
  );
}
