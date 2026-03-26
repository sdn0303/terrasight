"use client";

import { useTranslations } from "next-intl";
import { useAnalysisStore, type WeightPreset } from "@/stores/analysis-store";

const PRESETS: WeightPreset[] = ["balance", "investment", "residential", "disaster"];

export function WeightPresetSelector() {
  const t = useTranslations("analyze.weightPreset");
  const weightPreset = useAnalysisStore((s) => s.weightPreset);
  const setWeightPreset = useAnalysisStore((s) => s.setWeightPreset);

  return (
    <div className="px-4 py-2">
      <div className="text-[9px] font-mono tracking-wider text-neutral-500 mb-1.5">WEIGHT PRESET</div>
      <div className="flex gap-1 flex-wrap">
        {PRESETS.map((preset) => (
          <button
            key={preset}
            type="button"
            onClick={() => setWeightPreset(preset)}
            className={`px-2 py-1 rounded text-[10px] font-mono transition-colors ${
              weightPreset === preset
                ? "bg-white/10 text-cyan-400"
                : "bg-neutral-800 text-neutral-500 hover:text-neutral-300"
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
