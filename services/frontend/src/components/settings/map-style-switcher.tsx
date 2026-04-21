"use client";

import { clsx } from "clsx";
import { useTranslation } from "@/lib/i18n";
import { type BaseMap, useUIStore } from "@/stores/ui-store";

const STYLES: { value: BaseMap; labelKey: string }[] = [
  { value: "dark", labelKey: "settings.mapStyle.dark" },
  { value: "satellite", labelKey: "settings.mapStyle.satellite" },
];

export function MapStyleSwitcher() {
  const baseMap = useUIStore((s) => s.baseMap);
  const setBaseMap = useUIStore((s) => s.setBaseMap);
  const { t } = useTranslation();

  return (
    <div className="space-y-2">
      <fieldset className="border-0 p-0 m-0">
        <legend
          className="text-xs font-medium uppercase tracking-wider"
          style={{ color: "var(--panel-text-secondary)" }}
        >
          {t("settings.mapStyle")}
        </legend>
        <div className="flex gap-1 mt-2">
          {STYLES.map(({ value, labelKey }) => (
            <button
              key={value}
              type="button"
              onClick={() => setBaseMap(value)}
              aria-pressed={baseMap === value}
              className={clsx(
                "flex-1 rounded-lg px-3 py-1.5 text-sm transition-colors",
                baseMap === value
                  ? "font-medium"
                  : "hover:bg-[var(--panel-hover-bg)]",
              )}
              style={{
                backgroundColor:
                  baseMap === value ? "var(--panel-active-bg)" : undefined,
                color: "var(--panel-text-primary)",
              }}
            >
              {t(labelKey)}
            </button>
          ))}
        </div>
      </fieldset>
    </div>
  );
}
