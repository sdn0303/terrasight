"use client";

import { X } from "lucide-react";
import { useUIStore } from "@/stores/ui-store";
import { LandPriceSection } from "./sections/LandPriceSection";
import { PopulationSection } from "./sections/PopulationSection";
import { TLSSection } from "./sections/TLSSection";

/**
 * 楽待ち型詳細パネル。エリアバッジまたは駅ドットクリック時に表示。
 * Ref: DESIGN.md Sec 5.3
 */
export function DetailPanel() {
  const selectedArea = useUIStore((s) => s.selectedArea);
  const setSelectedArea = useUIStore((s) => s.setSelectedArea);

  if (!selectedArea) return null;

  return (
    <div
      className="fixed overflow-y-auto animate-slide-in-left"
      style={{
        left: "calc(var(--ts-sidebar-width) + var(--ts-gap-panel) * 2)",
        top: "calc(var(--ts-tab-height) + var(--ts-gap-panel) * 2 + 4px)",
        width: 340,
        maxHeight:
          "calc(100vh - var(--ts-tab-height) - var(--ts-gap-panel) * 3 - 16px)",
        background: "var(--ts-bg-panel)",
        borderRadius: "var(--ts-panel-radius)",
        zIndex: 20,
      }}
    >
      {/* Header */}
      <div className="flex items-start justify-between p-4">
        <div>
          <h2
            className="text-base font-medium"
            style={{ color: "var(--ts-text-primary)" }}
          >
            {selectedArea.name || selectedArea.code}
          </h2>
          <p
            className="text-xs mt-0.5"
            style={{ color: "var(--ts-text-muted)" }}
          >
            {selectedArea.level === "prefecture" ? "都道府県" : "市区町村"}
          </p>
        </div>
        <button
          type="button"
          onClick={() => setSelectedArea(null)}
          className="p-1 rounded hover:bg-white/10 transition-colors"
          aria-label="閉じる"
        >
          <X size={16} style={{ color: "var(--ts-text-muted)" }} />
        </button>
      </div>

      {/* Divider */}
      <Divider />

      {/* TLS Score Section */}
      <TLSSection />

      <Divider />

      {/* Land Price Section */}
      <LandPriceSection />

      <Divider />

      {/* Population Section */}
      <PopulationSection areaCode={selectedArea.code} />
    </div>
  );
}

function Divider() {
  return (
    <div
      className="mx-4"
      style={{
        height: 1,
        background: "var(--ts-border-divider)",
      }}
    />
  );
}
