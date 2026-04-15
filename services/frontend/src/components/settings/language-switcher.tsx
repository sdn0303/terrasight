"use client";

import { clsx } from "clsx";
import { useTranslation } from "@/lib/i18n";
import { useUIStore } from "@/stores/ui-store";

type Locale = "ja" | "en";

const LOCALES: { value: Locale; labelKey: string }[] = [
  { value: "ja", labelKey: "settings.language.ja" },
  { value: "en", labelKey: "settings.language.en" },
];

export function LanguageSwitcher() {
  const locale = useUIStore((s) => s.locale);
  const setLocale = useUIStore((s) => s.setLocale);
  const { t } = useTranslation();

  return (
    <div className="space-y-2">
      <fieldset className="border-0 p-0 m-0">
        <legend
          className="text-xs font-medium uppercase tracking-wider"
          style={{ color: "var(--panel-text-secondary)" }}
        >
          {t("settings.language")}
        </legend>
        <div className="flex gap-1 mt-2">
          {LOCALES.map(({ value, labelKey }) => (
            <button
              key={value}
              type="button"
              onClick={() => setLocale(value)}
              aria-pressed={locale === value}
              className={clsx(
                "flex-1 rounded-lg px-3 py-1.5 text-sm transition-colors",
                locale === value
                  ? "font-medium"
                  : "hover:bg-[var(--panel-hover-bg)]",
              )}
              style={{
                backgroundColor:
                  locale === value ? "var(--panel-active-bg)" : undefined,
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
