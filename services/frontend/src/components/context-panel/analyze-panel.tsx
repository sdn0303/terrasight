"use client";

import { useTranslations } from "next-intl";
import { TlsScoreHeader } from "@/components/analyze/tls-score-header";
import { AxisBarList } from "@/components/analyze/axis-bar-list";
import { WeightPresetSelector } from "@/components/analyze/weight-preset-selector";
import { CrossAnalysis } from "@/components/analyze/cross-analysis";
import { useScore } from "@/features/score/api/use-score";
import { useMapStore } from "@/stores/map-store";
import { useUIStore } from "@/stores/ui-store";

export function AnalyzePanel() {
  const t = useTranslations();
  const analysisPoint = useMapStore((s) => s.analysisPoint);
  const setMode = useUIStore((s) => s.setMode);

  const lat = analysisPoint?.lat ?? null;
  const lng = analysisPoint?.lng ?? null;
  const { data: score, isLoading } = useScore(lat, lng);

  if (!analysisPoint) {
    return (
      <div className="flex items-center justify-center h-full px-4">
        <div className="text-xs text-center text-neutral-500">{t("explore.prompt")}</div>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="p-4 space-y-3">
        <div className="h-4 w-32 bg-neutral-800 rounded animate-pulse" />
        <div className="h-20 w-full bg-neutral-800 rounded animate-pulse" />
        <div className="h-32 w-full bg-neutral-800 rounded animate-pulse" />
      </div>
    );
  }

  if (!score) return null;

  return (
    <div className="flex flex-col h-full overflow-y-auto">
      <div className="px-4 pt-4 pb-1">
        <div className="text-[9px] font-mono tracking-widest text-cyan-400">ANALYZE</div>
        <div className="text-xs mt-1 text-neutral-300">
          {analysisPoint.address ?? `${lat?.toFixed(4)}, ${lng?.toFixed(4)}`}
        </div>
      </div>

      <TlsScoreHeader score={score.tls.score} grade={score.tls.grade} label={score.tls.label} />
      <WeightPresetSelector />

      <div className="border-t border-neutral-800 my-2" />
      <AxisBarList axes={score.axes} />

      <div className="border-t border-neutral-800 my-2" />
      <CrossAnalysis crossAnalysis={score.cross_analysis} />

      <div className="px-4 py-3 mt-auto">
        <button
          type="button"
          onClick={() => setMode("compare")}
          className="w-full rounded-lg py-2 text-xs bg-neutral-800 text-neutral-400 border border-neutral-700 hover:text-neutral-200 transition-colors"
        >
          {t("analyze.toCompare")}
        </button>
      </div>

      <div className="px-4 pb-3">
        <div className="text-[9px] text-neutral-600">{score.metadata.disclaimer}</div>
      </div>
    </div>
  );
}
