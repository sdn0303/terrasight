"use client";

import { useEffect } from "react";
import { useTranslations } from "next-intl";
import { THEMES, getLayerIdsForThemes } from "@/lib/themes";
import type { ThemeId } from "@/lib/themes";
import { useUIStore } from "@/stores/ui-store";
import { useMapStore } from "@/stores/map-store";

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
    <div className="grid grid-cols-2 gap-2 px-4 py-3">
      {THEMES.map((theme) => {
        const isActive = activeThemes.has(theme.id);
        return (
          <button
            key={theme.id}
            type="button"
            onClick={() => toggleTheme(theme.id)}
            className={`flex items-center gap-2 rounded-lg px-3 py-2.5 text-xs transition-colors border ${
              isActive
                ? "bg-white/5 border-cyan-500/50 text-cyan-400"
                : "bg-neutral-800/50 border-transparent text-neutral-500 hover:text-neutral-300"
            }`}
            aria-pressed={isActive}
          >
            <span>{ICONS[theme.id]}</span>
            <span>{t(theme.labelKey)}</span>
          </button>
        );
      })}
    </div>
  );
}
