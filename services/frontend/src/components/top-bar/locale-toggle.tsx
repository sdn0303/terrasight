"use client";

import { useUIStore } from "@/stores/ui-store";

export function LocaleToggle() {
  const locale = useUIStore((s) => s.locale);
  const setLocale = useUIStore((s) => s.setLocale);

  return (
    <button
      type="button"
      onClick={() => setLocale(locale === "ja" ? "en" : "ja")}
      className="px-2 py-1 rounded text-[10px] font-mono tracking-wider border border-ds-border-primary text-ds-text-muted hover:text-ds-text-primary hover:border-ds-border-primary transition-colors"
      aria-label={`Switch to ${locale === "ja" ? "English" : "Japanese"}`}
    >
      {locale === "ja" ? "EN" : "JA"}
    </button>
  );
}
