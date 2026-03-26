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
    <div className="flex gap-1" role="tablist" aria-label="Application mode">
      {MODES.map((m) => (
        <button
          key={m}
          type="button"
          role="tab"
          aria-selected={mode === m}
          onClick={() => setMode(m)}
          className={`px-3 py-1.5 rounded text-xs font-mono tracking-wide transition-colors ${
            mode === m
              ? "bg-white/10 text-cyan-400"
              : "text-neutral-500 hover:text-neutral-300"
          }`}
        >
          {t(m)}
        </button>
      ))}
    </div>
  );
}
