"use client";

import { Popup } from "react-map-gl/maplibre";
import { FinderPanel } from "@/components/finder/finder-panel";
import { CompareTab } from "@/components/insight/compare-tab";
import { InfraTab } from "@/components/insight/infra-tab";
import { IntelTab } from "@/components/insight/intel-tab";
import { RiskTab } from "@/components/insight/risk-tab";
import { TrendTab } from "@/components/insight/trend-tab";
import { LayerControlPanel } from "@/components/layer/layer-control-panel";
import type { DrawerTabDef } from "@/components/layout/insight-drawer";
import { InsightDrawer } from "@/components/layout/insight-drawer";
import { MapCanvasFrame } from "@/components/layout/map-canvas-frame";
import { SidebarRail } from "@/components/layout/sidebar-rail";
import { LayerRenderer } from "@/components/map/layer-renderer";
import { MapView } from "@/components/map/map-view";
import { PopupCard } from "@/components/map/popup-card";
import { OpportunitiesSheet } from "@/components/opportunities/opportunities-sheet";
import { ThemesPanel } from "@/components/theme/themes-panel";
import { useMapInteraction } from "@/hooks/use-map-interaction";
import { useMapPage } from "@/hooks/use-map-page";
import { useMapStore } from "@/stores/map-store";
import { useUIStore } from "@/stores/ui-store";

export default function Home() {
  const { handleFeatureClick } = useMapInteraction();
  const page = useMapPage();

  const insight = useUIStore((s) => s.insight);
  const activeTab = useUIStore((s) => s.activeTab);
  const setActiveTab = useUIStore((s) => s.setActiveTab);
  const setInsight = useUIStore((s) => s.setInsight);
  const leftPanel = useUIStore((s) => s.leftPanel);
  const setLeftPanel = useUIStore((s) => s.setLeftPanel);
  const bottomSheet = useUIStore((s) => s.bottomSheet);
  const setBottomSheet = useUIStore((s) => s.setBottomSheet);
  const comparePoints = useUIStore((s) => s.comparePoints);

  const insightOpen = insight !== null;
  const insightLat = insight?.lat ?? null;
  const insightLng = insight?.lng ?? null;

  const insightTitle =
    insight?.kind === "point"
      ? `${insight.lat.toFixed(4)}, ${insight.lng.toFixed(4)}`
      : insight?.kind === "property"
        ? insight.id
        : "";

  const baseDrawerTabs: DrawerTabDef[] =
    insightLat !== null && insightLng !== null
      ? [
          {
            id: "intel",
            label: "Intel",
            content: <IntelTab lat={insightLat} lng={insightLng} />,
          },
          {
            id: "trend",
            label: "Trend",
            content: <TrendTab lat={insightLat} lng={insightLng} />,
          },
          {
            id: "risk",
            label: "Risk",
            content: <RiskTab lat={insightLat} lng={insightLng} />,
          },
          {
            id: "infra",
            label: "Infra",
            content: <InfraTab lat={insightLat} lng={insightLng} />,
          },
        ]
      : [];

  // Phase 6: the Compare tab is a conditional 5th drawer tab that only
  // appears when the user has ticked at least 2 rows in the opportunities
  // sheet. It is tied to `insight !== null` so the drawer still opens via
  // a row click; opening the drawer from compare-only mode is deferred to
  // a follow-up PR (see docs/designs/2026-04-12-known-issues-frontend.md).
  const drawerTabs: DrawerTabDef[] =
    baseDrawerTabs.length > 0 && comparePoints.length >= 2
      ? [
          ...baseDrawerTabs,
          { id: "compare", label: "Compare", content: <CompareTab /> },
        ]
      : baseDrawerTabs;

  return (
    <MapCanvasFrame aria-label="Terrasight map canvas">
      <div className="absolute inset-0">
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
            className="pointer-events-none absolute inset-0 z-10 flex items-center justify-center"
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

      <SidebarRail />

      <LayerControlPanel
        open={leftPanel === "layers"}
        onClose={() => setLeftPanel(null)}
      />
      <ThemesPanel
        open={leftPanel === "themes"}
        onClose={() => setLeftPanel(null)}
      />
      <FinderPanel
        open={leftPanel === "finder"}
        onClose={() => setLeftPanel(null)}
        onSearch={() => setBottomSheet("opportunities")}
        matchCount={1247}
      />

      <OpportunitiesSheet
        open={bottomSheet === "opportunities"}
        onClose={() => setBottomSheet(null)}
      />

      <InsightDrawer
        open={insightOpen}
        onClose={() => setInsight(null)}
        title={insightTitle}
        subtitle="Selected on map"
        tabs={drawerTabs}
        activeTab={activeTab}
        onTabChange={setActiveTab}
      />

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
    </MapCanvasFrame>
  );
}
