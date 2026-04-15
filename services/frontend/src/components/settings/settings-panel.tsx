"use client";

import { useTranslation } from "@/lib/i18n";
import { LanguageSwitcher } from "./language-switcher";
import { MapStyleSwitcher } from "./map-style-switcher";

interface SettingsPanelProps {
  open: boolean;
  onClose: () => void;
}

export function SettingsPanel({ open, onClose }: SettingsPanelProps) {
  const { t } = useTranslation();

  if (!open) return null;

  return (
    <>
      {/* Backdrop: closes panel on outside click */}
      <div
        className="fixed inset-0 z-40"
        aria-hidden="true"
        onClick={onClose}
      />

      <div
        role="dialog"
        aria-modal="true"
        aria-label={t("settings.title")}
        className="absolute bottom-4 left-[calc(100%+8px)] z-50 w-[280px] rounded-xl p-4 shadow-lg"
        style={{
          backgroundColor: "var(--panel-bg)",
          border: "1px solid var(--panel-border)",
        }}
      >
        <div className="space-y-5">
          <MapStyleSwitcher />
          <LanguageSwitcher />
        </div>
      </div>
    </>
  );
}
