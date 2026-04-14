

import { DATA_MODES, useDataModeStore } from "@/stores/data-mode-store";
import type { NavigationLevel } from "@/hooks/use-navigation-level";

interface DataModeBarProps {
  level: NavigationLevel;
}

export function DataModeBar({ level }: DataModeBarProps) {
  const mode = useDataModeStore((s) => s.mode);
  const setMode = useDataModeStore((s) => s.setMode);

  // L3/L4 では非表示
  if (level === "L3" || level === "L4") return null;

  return (
    <div
      className="absolute top-2 left-1/2 -translate-x-1/2 z-10 flex gap-1 bg-black/80 backdrop-blur-sm rounded-xl px-2 py-1"
      role="tablist"
      aria-label="データモード選択"
    >
      {DATA_MODES.map((dm) => (
        <button
          key={dm.id}
          type="button"
          role="tab"
          onClick={() => {
            if (dm.available) {
              setMode(dm.id);
            }
          }}
          disabled={!dm.available}
          aria-selected={mode === dm.id}
          aria-current={mode === dm.id ? "true" : undefined}
          className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
            mode === dm.id
              ? "bg-white/20 text-white"
              : dm.available
                ? "text-white/60 hover:text-white hover:bg-white/10"
                : "text-white/30 cursor-not-allowed"
          }`}
        >
          {dm.labelJa}
          {!dm.available && (
            <span className="ml-1 text-[10px] text-amber-400/80">Soon</span>
          )}
        </button>
      ))}
    </div>
  );
}
