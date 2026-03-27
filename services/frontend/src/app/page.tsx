"use client";

import { ComparePanel } from "@/components/context-panel/compare-panel";
import { ContextPanel } from "@/components/context-panel/context-panel";
import { ExplorePanel } from "@/components/context-panel/explore-panel";
import { LayerRenderer } from "@/components/map/layer-renderer";
import { MapView } from "@/components/map/map-view";
import { PopupCard } from "@/components/map/popup-card";
import { StatusBar } from "@/components/status-bar";
import { TopBar } from "@/components/top-bar/top-bar";
import { useMapInteraction } from "@/hooks/use-map-interaction";
import { useMapPage } from "@/hooks/use-map-page";
import {
  PANEL_WIDTH,
  STATUS_BAR_HEIGHT,
  TOP_BAR_HEIGHT,
} from "@/lib/constants";
import { useUIStore } from "@/stores/ui-store";

export default function Home() {
  const mode = useUIStore((s) => s.mode);
  const { handleFeatureClick } = useMapInteraction();
  const page = useMapPage();

  return (
    <div className="relative h-screen w-screen overflow-hidden">
      <TopBar />

      <ContextPanel>
        {mode === "explore" && <ExplorePanel />}
        {mode === "compare" && <ComparePanel />}
      </ContextPanel>

      <div
        className="absolute"
        style={{
          top: TOP_BAR_HEIGHT,
          left: PANEL_WIDTH,
          right: 0,
          bottom: STATUS_BAR_HEIGHT,
        }}
      >
        <MapView
          onMoveEnd={page.handleMoveEnd}
          onFeatureClick={handleFeatureClick}
        >
          <LayerRenderer
            visibleLayers={page.visibleLayers}
            staticLayers={page.staticLayers}
            apiLayers={page.apiLayers}
            areaData={page.areaData as Record<string, unknown> | null}
            landPriceData={page.landPriceData}
            isLandPriceFetching={page.isLandPriceFetching}
            isLandPriceError={page.isLandPriceError}
            isZoomTooLow={page.isZoomTooLow}
            populationYear={page.populationYear}
            setPopulationYear={page.setPopulationYear}
            landPriceYear={page.landPriceYear}
            setLandPriceYear={page.setLandPriceYear}
            landPriceFeatureCount={page.landPriceData.features.length}
          />
        </MapView>
        {page.isZoomTooLow && (
          <div
            className="absolute inset-0 flex items-center justify-center pointer-events-none"
            style={{ zIndex: 10 }}
          >
            <div
              className="rounded-lg px-6 py-3 text-center"
              style={{
                background: "rgba(12, 12, 20, 0.75)",
                border: "1px solid var(--border-primary)",
                fontFamily: "var(--font-mono)",
                fontSize: "12px",
                color: "var(--text-secondary)",
              }}
            >
              ズームインしてデータを表示
            </div>
          </div>
        )}
      </div>

      {page.selectedFeature && page.selectedLayerConfig?.popupFields && (
        <div
          className="fixed z-30 pointer-events-none"
          style={{
            top: "50%",
            left: "50%",
            transform: "translate(-50%, -50%)",
          }}
        >
          <div className="pointer-events-auto">
            <PopupCard
              layerNameJa={page.selectedLayerConfig.nameJa}
              fields={page.selectedLayerConfig.popupFields}
              properties={page.selectedFeature.properties}
            />
          </div>
        </div>
      )}

      <StatusBar
        lat={page.viewState.latitude}
        lng={page.viewState.longitude}
        zoom={page.viewState.zoom}
        isLoading={page.isLoading}
        isDemoMode={page.isDemoMode}
        truncatedLayers={page.truncatedLayers}
        wasmError={page.wasmError}
        areaDataError={page.areaDataError}
        isZoomTooLow={page.isZoomTooLow}
      />
    </div>
  );
}
