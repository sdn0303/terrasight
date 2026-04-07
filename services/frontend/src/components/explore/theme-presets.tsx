"use client";

import type { LucideIcon } from "lucide-react";
import { Banknote, Home, Shield, TrendingUp } from "lucide-react";
import { useTranslations } from "next-intl";
import { useEffect } from "react";
import type { ThemeId } from "@/lib/themes";
import { getLayerIdsByTheme, getLayerIdsForThemes, THEMES } from "@/lib/themes";
import { useMapStore } from "@/stores/map-store";
import { useUIStore } from "@/stores/ui-store";

const ICONS: Record<ThemeId, LucideIcon> = {
  safety: Shield,
  livability: Home,
  price: Banknote,
  future: TrendingUp,
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
        const Icon = ICONS[theme.id];
        return (
          <button
            key={theme.id}
            type="button"
            onClick={() => toggleTheme(theme.id)}
            className={`flex items-start gap-3 rounded-lg px-4 py-3 text-left transition-colors border ${
              isActive
                ? "bg-ds-hover-accent border-ds-accent-primary/50"
                : "bg-ds-bg-tertiary/50 border-transparent hover:bg-ds-bg-tertiary"
            }`}
            aria-pressed={isActive}
          >
            <div
              className={`w-8 h-8 rounded-lg flex items-center justify-center flex-shrink-0 ${
                isActive ? "bg-ds-accent-primary/10" : "bg-ds-bg-tertiary/50"
              }`}
            >
              <Icon
                size={18}
                style={{
                  color: isActive
                    ? "var(--accent-primary)"
                    : "var(--text-muted)",
                }}
              />
            </div>
            <div className="flex-1 min-w-0">
              <div className="flex items-center justify-between">
                <span
                  className={`text-xs font-medium ${
                    isActive ? "text-ds-accent-primary" : "text-ds-text-primary"
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
