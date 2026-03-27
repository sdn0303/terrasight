"use client";

import { useTranslations } from "next-intl";
import { useEffect } from "react";
import type { ThemeId } from "@/lib/themes";
import { getLayerIdsByTheme, getLayerIdsForThemes, THEMES } from "@/lib/themes";
import { useMapStore } from "@/stores/map-store";
import { useUIStore } from "@/stores/ui-store";

const ICONS: Record<ThemeId, string> = {
  safety: "\u{1F6E1}",
  livability: "\u{1F3D8}",
  price: "\u{1F4B0}",
  future: "\u{1F4C8}",
};

export function ThemePresets() {
  const t = useTranslations();
  const activeThemes = useUIStore((s) => s.activeThemes);
  const toggleTheme = useUIStore((s) => s.toggleTheme);

  useEffect(() => {
    if (activeThemes.size === 0) {
      useMapStore.setState({ visibleLayers: new Set<string>() });
      return;
    }
    const themeLayerIds = getLayerIdsForThemes(activeThemes);
    useMapStore.setState({ visibleLayers: themeLayerIds });
  }, [activeThemes]);

  return (
    <div className="flex flex-col gap-2 px-4 py-3">
      {THEMES.map((theme) => {
        const isActive = activeThemes.has(theme.id);
        const layerCount = getLayerIdsByTheme(theme.id).length;
        return (
          <button
            key={theme.id}
            type="button"
            onClick={() => toggleTheme(theme.id)}
            className={`flex items-start gap-3 rounded-lg px-4 py-3 text-left transition-colors border ${
              isActive
                ? "bg-ds-hover-accent border-ds-accent-cyan/50"
                : "bg-ds-bg-tertiary/50 border-transparent hover:bg-ds-bg-tertiary"
            }`}
            aria-pressed={isActive}
          >
            <span className="text-xl mt-0.5">{ICONS[theme.id]}</span>
            <div className="flex-1 min-w-0">
              <div className="flex items-center justify-between">
                <span
                  className={`text-xs font-medium ${
                    isActive ? "text-ds-accent-cyan" : "text-ds-text-primary"
                  }`}
                >
                  {t(theme.labelKey)}
                </span>
                <span
                  className="text-[9px] font-mono"
                  style={{ color: "var(--text-muted)" }}
                >
                  {layerCount} layers
                </span>
              </div>
              <p
                className="text-[10px] mt-0.5 leading-relaxed"
                style={{ color: "var(--text-secondary)" }}
              >
                {t(`theme.${theme.id}.desc`)}
              </p>
            </div>
          </button>
        );
      })}
    </div>
  );
}
