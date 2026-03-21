"use client";

import { MapView } from "@/components/map/map-view";
import { CRTOverlay } from "@/components/crt-overlay";
import { StatusBar } from "@/components/status-bar";
import { useMapStore } from "@/stores/map-store";

export default function Home() {
  const { viewState } = useMapStore();

  return (
    <div className="relative h-screen w-screen overflow-hidden">
      <MapView />
      <CRTOverlay />
      <StatusBar
        lat={viewState.latitude}
        lng={viewState.longitude}
        zoom={viewState.zoom}
        isLoading={false}
        isDemoMode={true}
      />
    </div>
  );
}
