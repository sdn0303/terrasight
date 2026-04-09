"use client";

import { useTranslations } from "next-intl";
import { DiffTable } from "@/components/compare/diff-table";
import { RadarComparison } from "@/components/compare/radar-chart";
import { useScore } from "@/features/score/api/use-score";
import { useUIStore } from "@/stores/ui-store";

const POINT_COLORS = [
  {
    label: "text-ds-accent-primary",
    bg: "bg-ds-accent-primary/10",
    score: "text-ds-accent-primary",
  },
  {
    label: "text-amber-400",
    bg: "bg-amber-400/10",
    score: "text-amber-400",
  },
  {
    label: "text-emerald-400",
    bg: "bg-emerald-400/10",
    score: "text-emerald-400",
  },
] as const;

export function ComparePanel() {
  const t = useTranslations("compare");
  const comparePoints = useUIStore((s) => s.comparePoints);
  const removeComparePoint = useUIStore((s) => s.removeComparePoint);
  const resetCompare = useUIStore((s) => s.resetCompare);
  const setMode = useUIStore((s) => s.setMode);

  // Fixed hook calls for all 3 slots — hooks must not be called conditionally
  const p0 = comparePoints[0];
  const p1 = comparePoints[1];
  const p2 = comparePoints[2];
  const { data: score0 } = useScore(p0?.lat ?? null, p0?.lng ?? null);
  const { data: score1 } = useScore(p1?.lat ?? null, p1?.lng ?? null);
  const { data: score2 } = useScore(p2?.lat ?? null, p2?.lng ?? null);
  const allScores = [score0, score1, score2];
  const scores = allScores.slice(0, comparePoints.length);
  const readyScores = scores.filter(
    (s): s is NonNullable<typeof s> => s != null,
  );

  return (
    <div className="flex flex-col h-full overflow-y-auto">
      <div className="px-4 pt-4 pb-2">
        <div className="text-[9px] font-semibold tracking-widest uppercase text-ds-accent-primary">
          COMPARE
        </div>
      </div>

      {/* Point cards */}
      <div className="flex flex-col px-4 gap-2 mb-3">
        {comparePoints.map((pt, i) => {
          const color = POINT_COLORS[i % POINT_COLORS.length]!;
          const score = scores[i];
          return (
            <div key={`pt-${i}`} className={`rounded-lg p-2 ${color.bg}`}>
              <div className="flex items-center justify-between">
                <div
                  className={`text-[9px] font-semibold tracking-wider uppercase ${color.label}`}
                >
                  Point {String.fromCharCode(65 + i)}
                </div>
                <button
                  type="button"
                  aria-label={`Remove point ${String.fromCharCode(65 + i)}`}
                  onClick={() => removeComparePoint(i)}
                  className="text-ds-text-muted hover:text-ds-text-primary text-xs"
                >
                  ×
                </button>
              </div>
              <div className="text-[10px] mt-0.5 text-ds-text-primary">
                {pt.address}
              </div>
              {score && (
                <div className={`text-lg font-bold mt-1 ${color.score}`}>
                  {Math.round(score.tls.score)}
                </div>
              )}
            </div>
          );
        })}
        {comparePoints.length < 3 && (
          <div className="rounded-lg p-2 bg-ds-bg-tertiary/30 border border-dashed border-ds-border-primary">
            <div className="text-[10px] text-ds-text-muted text-center py-1">
              Click map to add point{" "}
              {String.fromCharCode(65 + comparePoints.length)}
            </div>
          </div>
        )}
      </div>

      {/* Radar + Diff when 2+ scores ready */}
      {readyScores.length >= 2 ? (
        <>
          <RadarComparison axesList={readyScores.map((s) => s.axes)} />
          <DiffTable
            axesList={readyScores.map((s) => s.axes)}
            tlsScores={readyScores.map((s) => s.tls.score)}
          />
        </>
      ) : (
        <div className="px-4 py-8 text-center text-xs text-ds-text-muted">
          {comparePoints.length === 0
            ? "Click first point on map"
            : comparePoints.length === 1
              ? "Click second point to compare"
              : "Loading scores..."}
        </div>
      )}

      {/* End compare */}
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
