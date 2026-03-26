"use client";

import { useUIStore } from "@/stores/ui-store";

export function LocaleToggle() {
  const locale = useUIStore((s) => s.locale);
  const setLocale = useUIStore((s) => s.setLocale);

  return (
    <button
      type="button"
      onClick={() => setLocale(locale === "ja" ? "en" : "ja")}
      className="px-2 py-1 rounded text-[10px] font-mono tracking-wider border border-neutral-700 text-neutral-500 hover:text-neutral-300 hover:border-neutral-500 transition-colors"
      aria-label={`Switch to ${locale === "ja" ? "English" : "Japanese"}`}
    >
      {locale === "ja" ? "EN" : "JA"}
    </button>
  );
}
