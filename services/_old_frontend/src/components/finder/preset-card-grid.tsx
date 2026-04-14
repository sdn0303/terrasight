"use client";

import { GLOW_SHADOW, GRADIENT } from "@/lib/theme-tokens";
import { useFilterStore, type WeightPreset } from "@/stores/filter-store";

const PRESETS: { id: WeightPreset; emoji: string; label: string }[] = [
  { id: "balance", emoji: "⚖", label: "Balance" },
  { id: "investment", emoji: "📈", label: "Investment" },
  { id: "residential", emoji: "🏠", label: "Residential" },
  { id: "disaster", emoji: "🛡", label: "Disaster" },
];

export function PresetCardGrid() {
  const preset = useFilterStore((s) => s.preset);
  const setPreset = useFilterStore((s) => s.setPreset);

  return (
    <div className="grid grid-cols-2 gap-2">
      {PRESETS.map((p) => {
        const active = p.id === preset;
        return (
          <button
            key={p.id}
            type="button"
            aria-pressed={active}
            onClick={() => setPreset(p.id)}
            className="rounded-[12px] p-2.5 text-center text-[10px] font-extrabold transition-all"
            style={
              active
                ? {
                    background: GRADIENT.primary,
                    color: "#fff",
                    boxShadow: GLOW_SHADOW.primarySubtle,
                  }
                : {
                    background: "var(--neutral-50)",
                    color: "var(--neutral-600)",
                    border: "1px solid var(--neutral-100)",
                  }
            }
          >
            <span className="mr-1" aria-hidden="true">
              {p.emoji}
            </span>
            {p.label}
          </button>
        );
      })}
    </div>
  );
}
