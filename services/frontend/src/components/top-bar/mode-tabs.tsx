"use client";

import { useTranslations } from "next-intl";
import type { AppMode } from "@/stores/ui-store";
import { useUIStore } from "@/stores/ui-store";

const MODES: AppMode[] = ["explore", "compare"];

export function ModeTabs() {
  const t = useTranslations("mode");
  const mode = useUIStore((s) => s.mode);
  const setMode = useUIStore((s) => s.setMode);

  return (
    <div
      className="flex gap-0.5 rounded-full p-1"
      style={{ background: "var(--surface-elevated)" }}
      role="tablist"
      aria-label="Application mode"
    >
      {MODES.map((m) => (
        <button
          key={m}
          type="button"
          role="tab"
          aria-selected={mode === m}
          onClick={() => setMode(m)}
          className={`px-3.5 py-1 rounded-full text-xs font-medium tracking-wide transition-all cursor-pointer ${
            mode === m
              ? "bg-gradient-to-r from-indigo-600 to-indigo-500 text-white shadow-sm"
              : "text-ds-text-muted hover:text-ds-text-primary"
          }`}
        >
          {t(m)}
        </button>
      ))}
    </div>
  );
}
