"use client";

import { useTranslations } from "next-intl";
import { useMapStore, type WeightPreset } from "@/stores/map-store";

const PRESETS: WeightPreset[] = [
  "balance",
  "investment",
  "residential",
  "disaster",
];

export function WeightPresetSelector() {
  const t = useTranslations("analyze.weightPreset");
  const weightPreset = useMapStore((s) => s.weightPreset);
  const setWeightPreset = useMapStore((s) => s.setWeightPreset);

  return (
    <div className="px-4 py-2">
      <div className="text-[9px] font-semibold tracking-wider uppercase text-ds-text-muted mb-1.5">
        WEIGHT PRESET
      </div>
      <div className="flex gap-1 flex-wrap">
        {PRESETS.map((preset) => (
          <button
            key={preset}
            type="button"
            onClick={() => setWeightPreset(preset)}
            className={`px-2 py-1 rounded text-[10px] font-mono transition-colors ${
              weightPreset === preset
                ? "bg-ds-accent-primary/10 text-ds-accent-primary"
                : "bg-ds-bg-tertiary text-ds-text-muted hover:text-ds-text-primary"
            }`}
            aria-pressed={weightPreset === preset}
          >
            {t(preset)}
          </button>
        ))}
      </div>
    </div>
  );
}
