"use client";

import { X } from "lucide-react";
import { useEffect } from "react";
import { HazardDetail } from "@/components/detail/hazard-detail";
import { LandPriceDetail } from "@/components/detail/land-price-detail";
import { ScoreDetail } from "@/components/detail/score-detail";
import { StationDetail } from "@/components/detail/station-detail";
import { TransactionDetail } from "@/components/detail/transaction-detail";
import { LEFT_PANEL_WIDTH, SIDEBAR_COLLAPSED_W, SIDEBAR_EXPANDED_W } from "@/lib/layout";
import { THEMES, type ThemeId } from "@/lib/theme-definitions";
import { useUIStore } from "@/stores/ui-store";

type DetailComponentProps = {
  lat: number;
  lng: number;
  featureProperties?: Record<string, unknown>;
};

const DETAIL_COMPONENTS: Record<
  ThemeId,
  React.ComponentType<DetailComponentProps>
> = {
  "land-price": LandPriceDetail,
  hazard: HazardDetail,
  transactions: TransactionDetail,
  station: StationDetail,
  score: ScoreDetail,
};

export function LeftDetailPanel() {
  const leftPanel = useUIStore((s) => s.leftPanel);
  const closeLeftPanel = useUIStore((s) => s.closeLeftPanel);
  const setLeftPanelTab = useUIStore((s) => s.setLeftPanelTab);
  const sidebarCollapsed = useUIStore((s) => s.sidebarCollapsed);

  const sidebarWidth = sidebarCollapsed ? SIDEBAR_COLLAPSED_W : SIDEBAR_EXPANDED_W;
  const isOpen = leftPanel !== null;

  useEffect(() => {
    if (!isOpen) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") closeLeftPanel();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [isOpen, closeLeftPanel]);

  if (!isOpen) return null;

  const { data, activeTab } = leftPanel;
  const DetailComponent = DETAIL_COMPONENTS[activeTab];

  return (
    <aside
      aria-label="地点詳細パネル"
      className="absolute z-20 flex flex-col overflow-hidden"
      style={{
        top: 0,
        left: sidebarWidth,
        bottom: 0,
        width: LEFT_PANEL_WIDTH,
        background: "var(--panel-bg)",
        borderRight: "1px solid var(--panel-border)",
        boxShadow: "4px 0 12px rgba(0,0,0,0.08)",
        transform: isOpen ? "translateX(0)" : "translateX(-100%)",
        transition: "transform 0.3s ease",
      }}
    >
      {/* Header */}
      <header
        className="flex items-start justify-between border-b px-4 py-3 flex-shrink-0"
        style={{ borderColor: "var(--panel-border)" }}
      >
        <div className="flex-1 min-w-0 pr-2">
          <p
            className="text-xs font-semibold truncate"
            style={{ color: "var(--panel-text-secondary)" }}
          >
            {data.address ?? `${data.lat.toFixed(5)}, ${data.lng.toFixed(5)}`}
          </p>
        </div>
        <button
          type="button"
          onClick={closeLeftPanel}
          aria-label="パネルを閉じる"
          className="flex h-7 w-7 items-center justify-center rounded-lg flex-shrink-0 transition-colors hover:bg-[var(--panel-hover-bg)]"
          style={{ color: "var(--panel-text-secondary)" }}
        >
          <X size={14} aria-hidden="true" />
        </button>
      </header>

      {/* Tab bar */}
      <nav
        className="flex border-b flex-shrink-0 overflow-x-auto"
        style={{ borderColor: "var(--panel-border)" }}
        aria-label="テーマタブ"
      >
        {THEMES.map((theme) => {
          const isActive = activeTab === theme.id;
          const Icon = theme.icon;
          return (
            <button
              key={theme.id}
              type="button"
              onClick={() => setLeftPanelTab(theme.id)}
              aria-current={isActive ? "page" : undefined}
              className="flex flex-1 flex-col items-center gap-0.5 px-1 py-2 text-[10px] font-medium transition-colors min-w-[52px]"
              style={{
                color: isActive
                  ? "var(--panel-text-primary)"
                  : "var(--panel-text-secondary)",
                borderBottom: isActive
                  ? "2px solid #6366f1"
                  : "2px solid transparent",
                background: isActive ? "var(--panel-active-bg)" : "transparent",
              }}
            >
              <Icon size={14} aria-hidden="true" />
              <span className="whitespace-nowrap">{theme.label}</span>
            </button>
          );
        })}
      </nav>

      {/* Detail content */}
      <div className="flex-1 overflow-y-auto px-4 py-4">
        <DetailComponent
          lat={data.lat}
          lng={data.lng}
          {...(data.featureProperties !== undefined && {
            featureProperties: data.featureProperties,
          })}
        />
      </div>
    </aside>
  );
}
