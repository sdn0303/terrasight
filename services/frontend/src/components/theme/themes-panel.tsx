"use client";

import {
  Banknote,
  Home,
  type LucideIcon,
  Shield,
  TrendingUp,
} from "lucide-react";
import { useTranslations } from "next-intl";
import { useEffect } from "react";
import { LeftPanel } from "@/components/layout/left-panel";
import {
  getLayerIdsByTheme,
  getLayerIdsForThemes,
  THEMES,
  type ThemeId,
} from "@/lib/themes";
import { useMapStore } from "@/stores/map-store";
import { useUIStore } from "@/stores/ui-store";
import { ThemeCard } from "./theme-card";

const ICONS: Record<ThemeId, LucideIcon> = {
  safety: Shield,
  livability: Home,
  price: Banknote,
  future: TrendingUp,
};

interface ThemesPanelProps {
  open: boolean;
  onClose: () => void;
}

export function ThemesPanel({ open, onClose }: ThemesPanelProps) {
  const t = useTranslations();
  const activeThemes = useUIStore((s) => s.activeThemes);
  const toggleTheme = useUIStore((s) => s.toggleTheme);
  const applyThemeLayers = useMapStore((s) => s.applyThemeLayers);

  // Sync map visibility whenever the active theme set changes, but only
  // while the panel is open so we do not stomp on manual toggles made
  // elsewhere after the panel is closed.
  useEffect(() => {
    if (!open) return;
    const themeLayerIds = getLayerIdsForThemes(activeThemes);
    applyThemeLayers(themeLayerIds);
  }, [open, activeThemes, applyThemeLayers]);

  return (
    <LeftPanel
      open={open}
      onClose={onClose}
      title="Investment Themes"
      subtitle={`${activeThemes.size} selected`}
      badge="THEMES"
    >
      <div className="grid grid-cols-2 gap-2">
        {THEMES.map((theme) => (
          <ThemeCard
            key={theme.id}
            id={theme.id}
            label={t(`theme.${theme.id}.name`)}
            description={t(`theme.${theme.id}.desc`)}
            layerCount={getLayerIdsByTheme(theme.id).length}
            icon={ICONS[theme.id]}
            active={activeThemes.has(theme.id)}
            onToggle={toggleTheme}
          />
        ))}
      </div>
    </LeftPanel>
  );
}
