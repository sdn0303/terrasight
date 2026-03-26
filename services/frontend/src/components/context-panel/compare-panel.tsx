"use client";

import { useTranslations } from "next-intl";
import { RadarComparison } from "@/components/compare/radar-chart";
import { DiffTable } from "@/components/compare/diff-table";
import { useScore } from "@/features/score/api/use-score";
import { useUIStore } from "@/stores/ui-store";

export function ComparePanel() {
  const t = useTranslations("compare");
  const comparePointA = useUIStore((s) => s.comparePointA);
  const comparePointB = useUIStore((s) => s.comparePointB);
  const resetCompare = useUIStore((s) => s.resetCompare);
  const setMode = useUIStore((s) => s.setMode);

  const { data: scoreA } = useScore(comparePointA?.lat ?? null, comparePointA?.lng ?? null);
  const { data: scoreB } = useScore(comparePointB?.lat ?? null, comparePointB?.lng ?? null);

  return (
    <div className="flex flex-col h-full overflow-y-auto">
      <div className="px-4 pt-4 pb-2">
        <div className="text-[9px] font-mono tracking-widest text-cyan-400">COMPARE</div>
      </div>

      {/* Point labels */}
      <div className="flex px-4 gap-2 mb-3">
        <div className="flex-1 rounded-lg p-2 bg-neutral-800/50">
          <div className="text-[9px] font-mono tracking-wider text-cyan-400">{t("pointA")}</div>
          <div className="text-[10px] mt-0.5 text-neutral-300">
            {comparePointA?.address ?? "Click map..."}
          </div>
          {scoreA && (
            <div className="text-lg font-bold mt-1 text-cyan-400">{Math.round(scoreA.tls.score)}</div>
          )}
        </div>
        <div className="flex-1 rounded-lg p-2 bg-neutral-800/50">
          <div className="text-[9px] font-mono tracking-wider text-amber-400">{t("pointB")}</div>
          <div className="text-[10px] mt-0.5 text-neutral-300">
            {comparePointB?.address ?? "Click map..."}
          </div>
          {scoreB && (
            <div className="text-lg font-bold mt-1 text-amber-400">{Math.round(scoreB.tls.score)}</div>
          )}
        </div>
      </div>

      {/* Radar + Diff Table */}
      {scoreA && scoreB ? (
        <>
          <RadarComparison axesA={scoreA.axes} axesB={scoreB.axes} />
          <DiffTable axesA={scoreA.axes} axesB={scoreB.axes} tlsA={scoreA.tls.score} tlsB={scoreB.tls.score} />
        </>
      ) : (
        <div className="px-4 py-8 text-center text-xs text-neutral-500">
          {!comparePointA ? "Click first point on map" : "Click second point on map"}
        </div>
      )}

      <div className="px-4 py-3 mt-auto">
        <button
          type="button"
          onClick={() => { resetCompare(); setMode("explore"); }}
          className="w-full rounded-lg py-2 text-xs bg-neutral-800 text-neutral-400 border border-neutral-700 hover:text-neutral-200 transition-colors"
        >
          {t("endCompare")}
        </button>
      </div>
    </div>
  );
}
