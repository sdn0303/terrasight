"use client";

import { useTranslations } from "next-intl";
import { DiffTable } from "@/components/compare/diff-table";
import { RadarComparison } from "@/components/compare/radar-chart";
import { useScore } from "@/features/score/api/use-score";
import { useUIStore } from "@/stores/ui-store";

export function ComparePanel() {
  const t = useTranslations("compare");
  const comparePointA = useUIStore((s) => s.comparePointA);
  const comparePointB = useUIStore((s) => s.comparePointB);
  const resetCompare = useUIStore((s) => s.resetCompare);
  const setMode = useUIStore((s) => s.setMode);

  const { data: scoreA } = useScore(
    comparePointA?.lat ?? null,
    comparePointA?.lng ?? null,
  );
  const { data: scoreB } = useScore(
    comparePointB?.lat ?? null,
    comparePointB?.lng ?? null,
  );

  return (
    <div className="flex flex-col h-full overflow-y-auto">
      <div className="px-4 pt-4 pb-2">
        <div className="text-[9px] font-semibold tracking-widest uppercase text-ds-accent-primary">
          COMPARE
        </div>
      </div>

      {/* Point labels */}
      <div className="flex px-4 gap-2 mb-3">
        <div className="flex-1 rounded-lg p-2 bg-ds-bg-tertiary/50">
          <div className="text-[9px] font-semibold tracking-wider uppercase text-ds-accent-primary">
            {t("pointA")}
          </div>
          <div className="text-[10px] mt-0.5 text-ds-text-primary">
            {comparePointA?.address ?? "Click map..."}
          </div>
          {scoreA && (
            <div className="text-lg font-bold mt-1 text-ds-accent-primary">
              {Math.round(scoreA.tls.score)}
            </div>
          )}
        </div>
        <div className="flex-1 rounded-lg p-2 bg-ds-bg-tertiary/50">
          <div className="text-[9px] font-mono tracking-wider text-amber-400">
            {t("pointB")}
          </div>
          <div className="text-[10px] mt-0.5 text-ds-text-primary">
            {comparePointB?.address ?? "Click map..."}
          </div>
          {scoreB && (
            <div className="text-lg font-bold mt-1 text-amber-400">
              {Math.round(scoreB.tls.score)}
            </div>
          )}
        </div>
      </div>

      {/* Radar + Diff Table */}
      {scoreA && scoreB ? (
        <>
          <RadarComparison axesA={scoreA.axes} axesB={scoreB.axes} />
          <DiffTable
            axesA={scoreA.axes}
            axesB={scoreB.axes}
            tlsA={scoreA.tls.score}
            tlsB={scoreB.tls.score}
          />
        </>
      ) : (
        <div className="px-4 py-8 text-center text-xs text-ds-text-muted">
          {!comparePointA
            ? "Click first point on map"
            : "Click second point on map"}
        </div>
      )}

      <div className="px-4 py-3 mt-auto">
        <button
          type="button"
          onClick={() => {
            resetCompare();
            setMode("explore");
          }}
          className="w-full rounded-lg py-2 text-xs bg-ds-bg-tertiary text-ds-text-secondary border border-ds-border-primary hover:text-ds-text-heading transition-colors"
        >
          {t("endCompare")}
        </button>
      </div>
    </div>
  );
}
