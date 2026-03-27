"use client";

import { useState } from "react";
import { useTranslations } from "next-intl";
import { ChevronRight } from "lucide-react";
import { ThemePresets } from "@/components/explore/theme-presets";
import { BreadcrumbNav } from "@/components/explore/breadcrumb-nav";
import { AreaCard } from "@/components/explore/area-card";
import { LayerSettings } from "@/components/shared/layer-settings";
import { useMapStore } from "@/stores/map-store";

export function ExplorePanel() {
  const t = useTranslations();
  const selectedArea = useMapStore((s) => s.selectedArea);
  const [layerSettingsOpen, setLayerSettingsOpen] = useState(false);

  return (
    <div className="flex flex-col h-full">
      <div className="px-4 pt-4 pb-2">
        <div className="text-[9px] font-mono tracking-widest text-ds-accent-cyan">
          {t("mode.explore").toUpperCase()}
        </div>
      </div>

      <BreadcrumbNav />
      <ThemePresets />

      {selectedArea ? (
        <AreaCard />
      ) : (
        <div className="px-4 py-8 text-center">
          <div className="text-xs text-ds-text-muted">
            {t("explore.prompt")}
          </div>
        </div>
      )}

      {/* Collapsed layer settings for power users */}
      <div className="mt-auto border-t border-ds-border-primary">
        <button
          type="button"
          onClick={() => setLayerSettingsOpen(!layerSettingsOpen)}
          className="flex items-center gap-2 w-full px-4 py-2 text-[9px] font-mono tracking-wider text-ds-text-muted hover:text-ds-text-primary"
        >
          <ChevronRight
            size={10}
            className="transition-transform"
            style={{ transform: layerSettingsOpen ? "rotate(90deg)" : "rotate(0deg)" }}
          />
          LAYER SETTINGS
        </button>
        {layerSettingsOpen && (
          <div className="max-h-[300px] overflow-y-auto">
            <LayerSettings />
          </div>
        )}
      </div>
    </div>
  );
}
