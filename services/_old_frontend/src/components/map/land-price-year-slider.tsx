"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { PRICE_LEGEND } from "@/features/land-prices/constants";
import { useMediaQuery } from "@/hooks/use-media-query";

interface LandPriceYearSliderProps {
  value: number;
  onChange: (year: number) => void;
  /** Only visible when land price time-series layer is active */
  visible: boolean;
  /** True while a fetch is in-flight — shows pulsing indicator and dims layers */
  isFetching?: boolean;
  /** True when the land price API call failed (404, network error, etc.) */
  isError?: boolean;
  /** Feature count from the current response; shows empty-state when 0 and not fetching */
  featureCount?: number;
  /** True when the map zoom level is below the minimum threshold for land price queries */
  isZoomTooLow?: boolean;
}

const MIN_YEAR = 2020;
const MAX_YEAR = 2024;
const STEP = 1;
const DEBOUNCE_MS = 200;
const ANIMATION_INTERVAL_MS = 800;
const YEARS = Array.from(
  { length: MAX_YEAR - MIN_YEAR + 1 },
  (_, i) => MIN_YEAR + i,
);

/**
 * Year slider for land price time-series temporal filtering.
 * Positioned bottom-left on the map to avoid overlap with the
 * population mesh slider (bottom-right).
 * Only renders when the land_price_ts layer is toggled ON.
 *
 * On mobile (< 768px) renders a full-width year button bar above the status bar.
 * On desktop renders the standard compact slider widget.
 */
export function LandPriceYearSlider({
  value,
  onChange,
  visible,
  isFetching = false,
  isError = false,
  featureCount,
  isZoomTooLow = false,
}: LandPriceYearSliderProps) {
  const [localValue, setLocalValue] = useState(value);
  const [isAnimating, setIsAnimating] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const animationRef = useRef<ReturnType<typeof setInterval> | undefined>(
    undefined,
  );
  const isMobile = useMediaQuery("(max-width: 767px)");

  useEffect(() => {
    setLocalValue(value);
  }, [value]);

  useEffect(() => {
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
      if (animationRef.current) clearInterval(animationRef.current);
    };
  }, []);

  const stopAnimation = useCallback(() => {
    if (animationRef.current) {
      clearInterval(animationRef.current);
      animationRef.current = undefined;
    }
    setIsAnimating(false);
  }, []);

  const startAnimation = useCallback(() => {
    if (animationRef.current) return;
    setIsAnimating(true);
    // Seed from current year so play starts from wherever the user left off
    let year = localValue;
    animationRef.current = setInterval(() => {
      year = year >= MAX_YEAR ? MIN_YEAR : year + 1;
      setLocalValue(year);
      onChange(year);
    }, ANIMATION_INTERVAL_MS);
  }, [localValue, onChange]);

  const togglePlayPause = useCallback(() => {
    if (isAnimating) {
      stopAnimation();
    } else {
      startAnimation();
    }
  }, [isAnimating, startAnimation, stopAnimation]);

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      // Manual drag stops animation
      if (animationRef.current) {
        clearInterval(animationRef.current);
        animationRef.current = undefined;
        setIsAnimating(false);
      }

      const newValue = Number(e.target.value);
      setLocalValue(newValue);

      if (timerRef.current) clearTimeout(timerRef.current);
      timerRef.current = setTimeout(() => {
        onChange(newValue);
      }, DEBOUNCE_MS);
    },
    [onChange],
  );

  const handleYearButtonClick = useCallback(
    (year: number) => {
      if (animationRef.current) {
        clearInterval(animationRef.current);
        animationRef.current = undefined;
        setIsAnimating(false);
      }
      setLocalValue(year);
      onChange(year);
    },
    [onChange],
  );

  if (!visible) return null;

  const showEmptyState =
    !isError &&
    featureCount !== undefined &&
    featureCount === 0 &&
    !isFetching &&
    !isZoomTooLow;
  const showZoomPrompt = isZoomTooLow && !isFetching && !isError;

  // Decision 7: mobile renders a full-width button bar above the status bar (28px)
  if (isMobile) {
    return (
      <fieldset
        aria-label="地価公示年度選択"
        style={{
          position: "absolute",
          bottom: 28,
          left: 0,
          right: 0,
          height: 44,
          background: "var(--bg-secondary)",
          borderTop: "1px solid var(--border-primary)",
          display: "flex",
          zIndex: 10,
        }}
      >
        {YEARS.map((year) => {
          const isActive = localValue === year;
          return (
            <button
              key={year}
              type="button"
              onClick={() => handleYearButtonClick(year)}
              aria-pressed={isActive}
              aria-label={`${year}年`}
              style={{
                flex: 1,
                minWidth: 44,
                height: "100%",
                border: "none",
                borderRight: "1px solid var(--border-primary)",
                background: isActive
                  ? "var(--accent-primary)"
                  : "var(--bg-tertiary)",
                color: isActive ? "var(--bg-primary)" : "var(--text-muted)",
                fontFamily: "var(--font-mono)",
                fontSize: 11,
                fontWeight: isActive ? 700 : 400,
                cursor: "pointer",
              }}
            >
              {year}
            </button>
          );
        })}
      </fieldset>
    );
  }

  // Desktop: compact slider widget
  return (
    <fieldset
      className="absolute bottom-[140px] left-[10px] xl:left-[290px] z-10 rounded border-none m-0"
      style={{
        background: "var(--bg-secondary)",
        border: "1px solid var(--border-primary)",
        minWidth: 200,
        padding: "10px 14px",
      }}
      aria-label="地価公示年度選択"
    >
      {/* Header label with play/pause button */}
      <div
        className="flex items-center justify-between mb-1"
        style={{ color: "var(--text-muted)", fontFamily: "var(--font-mono)" }}
      >
        <span className="text-[9px] tracking-[0.1em]">地価公示 LAND PRICE</span>
        <button
          type="button"
          onClick={togglePlayPause}
          disabled={isFetching || isError || isZoomTooLow}
          aria-label={isAnimating ? "アニメーション停止" : "アニメーション再生"}
          aria-pressed={isAnimating}
          className="text-[10px] leading-none ml-2 cursor-pointer disabled:cursor-not-allowed disabled:opacity-40"
          style={{
            background: "transparent",
            border: "1px solid var(--border-primary)",
            borderRadius: 3,
            color: isAnimating ? "var(--accent-primary)" : "var(--text-muted)",
            padding: "2px 6px",
            fontFamily: "var(--font-mono)",
          }}
        >
          {isAnimating ? "■" : "▶"}
        </button>
      </div>

      {/* Year display with Decision 2: pulsing cyan dot when fetching */}
      <div
        className="text-center text-lg font-bold mb-1 flex items-center justify-center gap-1.5"
        style={{
          color: "var(--layer-landprice)",
          fontFamily: "var(--font-mono)",
        }}
      >
        {localValue}
        {(isFetching || isAnimating) && (
          <span
            aria-hidden="true"
            style={{
              display: "inline-block",
              width: 6,
              height: 6,
              borderRadius: "50%",
              background: "var(--accent-primary)",
              animation: "lp-pulse 1.2s ease-in-out infinite",
              flexShrink: 0,
            }}
          />
        )}
      </div>

      {/* Decision 8: range input with aria-valuetext */}
      <input
        type="range"
        min={MIN_YEAR}
        max={MAX_YEAR}
        step={STEP}
        value={localValue}
        onChange={handleChange}
        className="w-full accent-[var(--layer-landprice)]"
        aria-label={`地価公示年度: ${localValue}年`}
        aria-valuemin={MIN_YEAR}
        aria-valuemax={MAX_YEAR}
        aria-valuenow={localValue}
        aria-valuetext={`${localValue}年`}
      />

      {/* Decision 8: visual tick marks at even intervals */}
      <div
        aria-hidden="true"
        style={{
          position: "relative",
          display: "flex",
          justifyContent: "space-between",
          marginTop: 2,
          paddingLeft: 4,
          paddingRight: 4,
        }}
      >
        {YEARS.map((year) => (
          <span
            key={year}
            style={{
              display: "inline-block",
              width: 2,
              height: 6,
              background: "var(--text-muted)",
            }}
          />
        ))}
      </div>

      {/* Year range labels */}
      <div
        className="flex justify-between text-[8px] mt-0.5"
        style={{ color: "var(--text-muted)", fontFamily: "var(--font-mono)" }}
      >
        <span>{MIN_YEAR}</span>
        <span>{MAX_YEAR}</span>
      </div>

      {/* Decision 4: compact color legend */}
      <div
        role="img"
        aria-label="地価カラースケール凡例"
        style={{
          display: "flex",
          justifyContent: "space-between",
          marginTop: 8,
          borderTop: "1px solid var(--border-primary)",
          paddingTop: 6,
        }}
      >
        {PRICE_LEGEND.map((stop) => (
          <div
            key={stop.color}
            style={{
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              gap: 2,
            }}
          >
            <span
              aria-hidden="true"
              style={{
                display: "inline-block",
                width: 8,
                height: 8,
                background: stop.color,
                borderRadius: 1,
              }}
            />
            <span
              style={{
                fontSize: 8,
                color: "var(--text-muted)",
                fontFamily: "var(--font-mono)",
              }}
            >
              {stop.label}
            </span>
          </div>
        ))}
      </div>

      {/* Error state — takes priority over empty state */}
      {isError && (
        <p
          role="alert"
          style={{
            margin: "6px 0 0",
            fontSize: 9,
            color: "var(--accent-danger)",
            fontFamily: "var(--font-mono)",
            textAlign: "center",
          }}
        >
          データ取得エラー
        </p>
      )}

      {/* Decision 3: empty-state message */}
      {showEmptyState && (
        <p
          role="status"
          aria-live="polite"
          style={{
            margin: "6px 0 0",
            fontSize: 9,
            color: "var(--text-muted)",
            fontFamily: "var(--font-mono)",
            textAlign: "center",
          }}
        >
          このエリアにデータなし
        </p>
      )}

      {showZoomPrompt && (
        <p
          role="status"
          aria-live="polite"
          style={{
            margin: "6px 0 0",
            fontSize: 9,
            color: "var(--text-muted)",
            fontFamily: "var(--font-mono)",
            textAlign: "center",
          }}
        >
          ズームインしてください
        </p>
      )}

      {/* Keyframe animation for the pulsing dot */}
      <style>{`
        @keyframes lp-pulse {
          0%, 100% { opacity: 1; transform: scale(1); }
          50% { opacity: 0.3; transform: scale(0.6); }
        }
      `}</style>
    </fieldset>
  );
}
