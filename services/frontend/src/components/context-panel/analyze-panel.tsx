"use client";

import { useTranslations } from "next-intl";
import { AxisBarList } from "@/components/analyze/axis-bar-list";
import { CrossAnalysis } from "@/components/analyze/cross-analysis";
import { InfraProximity } from "@/components/analyze/infra-proximity";
import { RiskBreakdown } from "@/components/analyze/risk-breakdown";
import { SinglePointRadar } from "@/components/analyze/single-point-radar";
import { TlsScoreHeader } from "@/components/analyze/tls-score-header";
import { TrendChart } from "@/components/analyze/trend-chart";
import { WeightPresetSelector } from "@/components/analyze/weight-preset-selector";
import { useScore } from "@/features/score/api/use-score";
import { useMapStore } from "@/stores/map-store";
import { useUIStore } from "@/stores/ui-store";

export function AnalyzePanel() {
  const t = useTranslations();
  const analysisPoint = useMapStore((s) => s.analysisPoint);
  const setMode = useUIStore((s) => s.setMode);

  const weightPreset = useMapStore((s) => s.weightPreset);
  const lat = analysisPoint?.lat ?? null;
  const lng = analysisPoint?.lng ?? null;
  const { data: score, isLoading } = useScore(lat, lng, weightPreset);

  if (!analysisPoint) {
    return (
      <div className="flex items-center justify-center h-full px-4">
        <div className="text-xs text-center text-ds-text-muted">
          {t("explore.prompt")}
        </div>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="p-4 space-y-3">
        <div className="h-4 w-32 bg-ds-bg-tertiary rounded animate-pulse" />
        <div className="h-20 w-full bg-ds-bg-tertiary rounded animate-pulse" />
        <div className="h-32 w-full bg-ds-bg-tertiary rounded animate-pulse" />
      </div>
    );
  }

  if (!score) return null;

  return (
    <div className="flex flex-col h-full overflow-y-auto">
      <div className="px-4 pt-4 pb-1">
        <div className="text-[9px] font-semibold tracking-widest uppercase text-ds-accent-primary">
          ANALYZE
        </div>
        <div className="text-xs mt-1 text-ds-text-primary">
          {analysisPoint.address ?? `${lat?.toFixed(4)}, ${lng?.toFixed(4)}`}
        </div>
      </div>

      <TlsScoreHeader
        score={score.tls.score}
        grade={score.tls.grade}
        label={score.tls.label}
      />
      <WeightPresetSelector />

      <div className="border-t border-ds-border-primary my-2" />
      <SinglePointRadar axes={score.axes} />

      <div className="border-t border-ds-border-primary my-2" />
      <TrendChart lat={lat!} lng={lng!} />

      <div className="border-t border-ds-border-primary my-2" />
      <RiskBreakdown disasterAxis={score.axes.disaster} />

      <div className="border-t border-ds-border-primary my-2" />
      <InfraProximity livabilityAxis={score.axes.livability} />

      <div className="border-t border-ds-border-primary my-2" />
      <AxisBarList axes={score.axes} />

      <div className="border-t border-ds-border-primary my-2" />
      <CrossAnalysis crossAnalysis={score.cross_analysis} />

      <div className="px-4 py-3 mt-auto">
        <button
          type="button"
          onClick={() => setMode("compare")}
          className="w-full rounded-lg py-2 text-xs bg-ds-bg-tertiary text-ds-text-secondary border border-ds-border-primary hover:text-ds-text-heading transition-colors"
        >
          {t("analyze.toCompare")}
        </button>
      </div>

      <div className="px-4 pb-3">
        <div className="text-[9px] text-ds-text-muted">
          {score.metadata.disclaimer}
        </div>
      </div>
    </div>
  );
}
