"use client";

import { AnimatePresence, motion } from "framer-motion";
import { ComponentBar } from "./component-bar";
import { ScoreGauge } from "./score-gauge";

type TlsGrade = "S" | "A" | "B" | "C" | "D" | "E";

function gradeColor(grade: TlsGrade): string {
  const map: Record<TlsGrade, string> = {
    S: "#10b981", // emerald
    A: "#22c55e", // green
    B: "#eab308", // yellow
    C: "#f97316", // orange
    D: "#ef4444", // red
    E: "#991b1b", // dark-red
  };
  return map[grade];
}

import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { Skeleton } from "@/components/ui/skeleton";
import { useScore } from "@/features/score/api/use-score";
import { useTrend } from "@/features/trend/api/use-trend";
import { useMediaQuery } from "@/hooks/use-media-query";
import { useMapStore } from "@/stores/map-store";
import { Sparkline } from "./sparkline";

/** Inner content shared between fixed panel and Sheet. */
function ScoreCardContent({ onClose }: { onClose: () => void }) {
  const { selectedFeature } = useMapStore();
  const lat = selectedFeature?.coordinates[1] ?? null;
  const lng = selectedFeature?.coordinates[0] ?? null;
  const { data: score, isLoading: scoreLoading } = useScore(lat, lng);
  const { data: trendData } = useTrend(lat, lng);

  if (!selectedFeature) return null;

  return (
    <div className="overflow-y-auto flex-1">
      {/* Header */}
      <div
        className="flex items-center justify-between px-4 py-3 border-b"
        style={{ borderColor: "var(--border-primary)" }}
      >
        <span
          className="text-[9px] tracking-[0.15em]"
          style={{ color: "var(--accent-primary)" }}
        >
          PROPERTY INTEL
        </span>
        <button
          type="button"
          onClick={onClose}
          className="text-xs"
          style={{ color: "var(--text-muted)" }}
          aria-label="Close score card"
        >
          &times;
        </button>
      </div>

      <div className="p-4 space-y-4">
        {/* Location */}
        <div>
          <div
            className="text-[9px] tracking-[0.15em] mb-1"
            style={{ color: "var(--text-muted)" }}
          >
            LOCATION
          </div>
          <div className="text-xs" style={{ color: "var(--text-primary)" }}>
            {String(
              selectedFeature.properties.address ??
                `${lat?.toFixed(4)}°N, ${lng?.toFixed(4)}°E`,
            )}
          </div>
        </div>

        {/* Investment Score */}
        {scoreLoading ? (
          <div className="space-y-2">
            <Skeleton className="h-4 w-32" />
            <Skeleton className="h-24 w-full" />
          </div>
        ) : score ? (
          <div
            className="rounded-lg p-3"
            style={{ background: "var(--bg-tertiary)" }}
          >
            <div
              className="text-[9px] tracking-[0.15em] mb-2"
              style={{ color: "var(--text-muted)" }}
            >
              INVESTMENT SCORE
            </div>
            <ScoreGauge score={score.tls.score} />
            <div className="flex items-center gap-1.5 mb-2">
              <span
                className="text-sm font-medium"
                style={{ color: gradeColor(score.tls.grade) }}
              >
                {score.tls.grade}
              </span>
              <span
                className="text-[10px]"
                style={{ color: "var(--text-muted)" }}
              >
                {score.tls.label}
              </span>
            </div>
            <div className="space-y-1.5 mt-3">
              <ComponentBar
                label="災害"
                value={score.axes.disaster.score}
                max={100}
                confidence={score.axes.disaster.confidence}
              />
              <ComponentBar
                label="地盤"
                value={score.axes.terrain.score}
                max={100}
                confidence={score.axes.terrain.confidence}
              />
              <ComponentBar
                label="利便性"
                value={score.axes.livability.score}
                max={100}
                confidence={score.axes.livability.confidence}
              />
              <ComponentBar
                label="将来性"
                value={score.axes.future.score}
                max={100}
                confidence={score.axes.future.confidence}
              />
              <ComponentBar
                label="価格"
                value={score.axes.price.score}
                max={100}
                confidence={score.axes.price.confidence}
              />
            </div>
          </div>
        ) : null}

        {/* Pricing */}
        {selectedFeature.properties.price_per_sqm !== undefined && (
          <div
            className="rounded-lg p-3"
            style={{ background: "var(--bg-tertiary)" }}
          >
            <div
              className="text-[9px] tracking-[0.15em] mb-2"
              style={{ color: "var(--text-muted)" }}
            >
              PRICING
            </div>
            <div className="flex justify-between text-xs">
              <span style={{ color: "var(--text-secondary)" }}>per sqm</span>
              <span style={{ color: "var(--accent-primary)" }}>
                ¥
                {Number(
                  selectedFeature.properties.price_per_sqm,
                ).toLocaleString()}
              </span>
            </div>
          </div>
        )}

        {/* Price Trend Sparkline */}
        {trendData && <Sparkline trend={trendData} />}

        {/* Disclaimer */}
        {score && (
          <div className="text-[9px]" style={{ color: "var(--text-muted)" }}>
            {score.metadata.disclaimer}
          </div>
        )}
      </div>
    </div>
  );
}

export function ScoreCard() {
  const { selectedFeature, selectFeature } = useMapStore();
  const isDesktop = useMediaQuery("(min-width: 1280px)");
  const isTablet = useMediaQuery("(min-width: 768px)");
  const isMobile = !isTablet;

  const handleClose = () => selectFeature(null);

  // Mobile: bottom Sheet
  if (isMobile) {
    return (
      <Sheet
        open={selectedFeature !== null}
        onOpenChange={(open) => {
          if (!open) handleClose();
        }}
      >
        <SheetContent
          side="bottom"
          showCloseButton={false}
          className="h-[70vh] p-0 flex flex-col"
          style={{
            background: "rgba(10, 10, 15, 0.97)",
            borderTop: "1px solid var(--border-primary)",
            color: "var(--text-primary)",
          }}
        >
          <SheetHeader className="sr-only">
            <SheetTitle>Property Score Card</SheetTitle>
          </SheetHeader>
          <ScoreCardContent onClose={handleClose} />
        </SheetContent>
      </Sheet>
    );
  }

  // Tablet and desktop: fixed right panel (280px tablet, 320px desktop)
  const panelWidth = isDesktop ? 320 : 280;

  return (
    <AnimatePresence>
      {selectedFeature && (
        <motion.aside
          initial={{ x: panelWidth }}
          animate={{ x: 0 }}
          exit={{ x: panelWidth }}
          transition={{ duration: 0.3 }}
          className="fixed right-4 top-4 bottom-[148px] overflow-y-auto rounded-lg"
          style={{
            width: panelWidth,
            background: "rgba(10, 10, 15, 0.9)",
            backdropFilter: "blur(12px)",
            border: "1px solid var(--border-primary)",
            zIndex: 50,
          }}
          aria-label="Property score card"
        >
          <ScoreCardContent onClose={handleClose} />
        </motion.aside>
      )}
    </AnimatePresence>
  );
}
