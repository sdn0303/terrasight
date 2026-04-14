import { Pause, Play } from "lucide-react";
import { useCallback, useEffect, useRef } from "react";
import { PageChip } from "@/components/layout/page-chip";
import { GLOW_SHADOW, GRADIENT } from "@/lib/theme-tokens";

interface TimeMachineChipProps {
  year: number;
  minYear: number;
  maxYear: number;
  isAnimating: boolean;
  onYearChange: (year: number) => void;
  onPlayToggle: () => void;
}

const ANIMATION_STEP_MS = 800;

export function TimeMachineChip({
  year,
  minYear,
  maxYear,
  isAnimating,
  onYearChange,
  onPlayToggle,
}: TimeMachineChipProps) {
  const intervalRef = useRef<number | null>(null);

  useEffect(() => {
    if (!isAnimating) {
      if (intervalRef.current !== null) {
        window.clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
      return;
    }
    intervalRef.current = window.setInterval(() => {
      const next = year >= maxYear ? minYear : year + 1;
      onYearChange(next);
    }, ANIMATION_STEP_MS);
    return () => {
      if (intervalRef.current !== null) {
        window.clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
    };
  }, [isAnimating, year, minYear, maxYear, onYearChange]);

  const handleSliderChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      onYearChange(Number(e.target.value));
    },
    [onYearChange],
  );

  return (
    <PageChip anchor="bottom-center" aria-label="Time machine">
      <div className="flex items-center gap-2.5">
        <button
          type="button"
          onClick={onPlayToggle}
          aria-label={isAnimating ? "Pause animation" : "Play animation"}
          className="flex h-7 w-7 items-center justify-center rounded-[9px] text-white"
          style={{
            background: GRADIENT.primary,
            boxShadow: GLOW_SHADOW.primarySubtle,
          }}
        >
          {isAnimating ? <Pause size={11} /> : <Play size={11} />}
        </button>
        <span
          className="text-[9px] font-extrabold uppercase"
          style={{
            color: "var(--neutral-400)",
            letterSpacing: "0.6px",
          }}
        >
          Year
        </span>
        <input
          type="range"
          min={minYear}
          max={maxYear}
          value={year}
          onChange={handleSliderChange}
          aria-label="Year selector"
          className="w-[160px]"
        />
        <span
          className="min-w-[36px] text-[12px] font-extrabold"
          style={{ color: "var(--neutral-900)" }}
        >
          {year}
        </span>
      </div>
    </PageChip>
  );
}
