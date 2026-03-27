"use client";

import { Popup } from "react-map-gl/maplibre";
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
import { useMapStore } from "@/stores/map-store";
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
          {page.selectedFeature && page.selectedLayerConfig?.popupFields && (
            <Popup
              longitude={page.selectedFeature.coordinates[0]}
              latitude={page.selectedFeature.coordinates[1]}
              anchor="bottom"
              closeOnClick={false}
              onClose={() => useMapStore.getState().selectFeature(null)}
              className="spatial-popup"
            >
              <PopupCard
                layerNameJa={page.selectedLayerConfig.nameJa}
                fields={page.selectedLayerConfig.popupFields}
                properties={page.selectedFeature.properties}
              />
            </Popup>
          )}
        </MapView>
        {page.isZoomTooLow && (
          <div
            className="absolute inset-0 z-10 flex items-center justify-center pointer-events-none"
            aria-hidden="true"
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

      <style>{`
        .spatial-popup .maplibregl-popup-content {
          background: transparent;
          padding: 0;
          box-shadow: none;
        }
        .spatial-popup .maplibregl-popup-tip {
          border-top-color: var(--bg-secondary);
        }
        .spatial-popup .maplibregl-popup-close-button {
          color: var(--text-muted);
          font-size: 16px;
          right: 4px;
          top: 2px;
        }
      `}</style>

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
