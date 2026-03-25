"use client";

import { AnimatePresence, motion } from "framer-motion";
import {
  PolarAngleAxis,
  PolarGrid,
  Radar,
  RadarChart,
  ResponsiveContainer,
} from "recharts";
import { useScore } from "@/features/score/api/use-score";
import { useUIStore } from "@/stores/ui-store";

export function ComparePanel() {
  const { compareMode, comparePointA, comparePointB, exitCompareMode } =
    useUIStore();
  const { data: scoreA } = useScore(
    comparePointA?.lat ?? null,
    comparePointA?.lng ?? null,
  );
  const { data: scoreB } = useScore(
    comparePointB?.lat ?? null,
    comparePointB?.lng ?? null,
  );

  const showPanel =
    compareMode && comparePointA !== null && comparePointB !== null;

  const radarData =
    scoreA && scoreB
      ? [
          {
            axis: "災害",
            A: scoreA.axes.disaster.score,
            B: scoreB.axes.disaster.score,
          },
          {
            axis: "地盤",
            A: scoreA.axes.terrain.score,
            B: scoreB.axes.terrain.score,
          },
          {
            axis: "利便性",
            A: scoreA.axes.livability.score,
            B: scoreB.axes.livability.score,
          },
          {
            axis: "将来性",
            A: scoreA.axes.future.score,
            B: scoreB.axes.future.score,
          },
          {
            axis: "価格",
            A: scoreA.axes.price.score,
            B: scoreB.axes.price.score,
          },
        ]
      : [];

  return (
    <AnimatePresence>
      {showPanel && (
        <motion.div
          initial={{ scale: 0.9, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          exit={{ scale: 0.9, opacity: 0 }}
          className="fixed inset-0 flex items-center justify-center"
          style={{ zIndex: 100 }}
        >
          {/* Backdrop */}
          <div
            className="absolute inset-0"
            style={{ background: "rgba(0,0,0,0.6)" }}
            onClick={exitCompareMode}
            onKeyDown={(e) => {
              if (e.key === "Escape" || e.key === "Enter" || e.key === " ") {
                exitCompareMode();
              }
            }}
            role="button"
            tabIndex={0}
            aria-label="Close comparison"
          />

          {/* Panel */}
          <div
            className="relative rounded-lg p-6 max-w-2xl w-full mx-4"
            style={{
              background: "var(--bg-secondary)",
              border: "1px solid var(--border-primary)",
              backdropFilter: "blur(12px)",
            }}
          >
            <div className="flex justify-between items-center mb-4">
              <span
                className="text-[9px] tracking-[0.15em]"
                style={{ color: "var(--accent-cyan)" }}
              >
                COMPARE ANALYSIS
              </span>
              <button
                type="button"
                onClick={exitCompareMode}
                className="text-sm"
                style={{ color: "var(--text-muted)" }}
                aria-label="Close comparison panel"
              >
                ×
              </button>
            </div>

            {/* Point labels */}
            <div className="flex justify-around mb-4">
              <div className="text-center">
                <div
                  className="text-[9px] tracking-[0.15em]"
                  style={{ color: "var(--accent-cyan)" }}
                >
                  POINT A
                </div>
                <div
                  className="text-xs"
                  style={{ color: "var(--text-primary)" }}
                >
                  {comparePointA?.address}
                </div>
                {scoreA && (
                  <div
                    className="text-lg font-bold"
                    style={{ color: "var(--accent-cyan)" }}
                  >
                    {Math.round(scoreA.tls.score)}
                  </div>
                )}
              </div>
              <div className="text-center">
                <div
                  className="text-[9px] tracking-[0.15em]"
                  style={{ color: "var(--accent-warning)" }}
                >
                  POINT B
                </div>
                <div
                  className="text-xs"
                  style={{ color: "var(--text-primary)" }}
                >
                  {comparePointB?.address}
                </div>
                {scoreB && (
                  <div
                    className="text-lg font-bold"
                    style={{ color: "var(--accent-warning)" }}
                  >
                    {Math.round(scoreB.tls.score)}
                  </div>
                )}
              </div>
            </div>

            {/* Radar chart */}
            {radarData.length > 0 && (
              <div style={{ width: "100%", height: 250 }}>
                <ResponsiveContainer>
                  <RadarChart data={radarData}>
                    <PolarGrid stroke="var(--border-primary)" />
                    <PolarAngleAxis
                      dataKey="axis"
                      tick={{ fill: "var(--text-secondary)", fontSize: 10 }}
                    />
                    <Radar
                      name="A"
                      dataKey="A"
                      stroke="var(--accent-cyan)"
                      fill="var(--accent-cyan)"
                      fillOpacity={0.2}
                    />
                    <Radar
                      name="B"
                      dataKey="B"
                      stroke="var(--accent-warning)"
                      fill="var(--accent-warning)"
                      fillOpacity={0.2}
                    />
                  </RadarChart>
                </ResponsiveContainer>
              </div>
            )}
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
