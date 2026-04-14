"use client";

import { useCallback, useEffect, useRef, useState } from "react";

interface YearSliderProps {
  value: number;
  onChange: (year: number) => void;
  /** Only visible when population mesh layer is active */
  visible: boolean;
}

const MIN_YEAR = 2020;
const MAX_YEAR = 2050;
const STEP = 5;
const DEBOUNCE_MS = 200;

/**
 * Year slider for population mesh temporal filtering.
 * Positioned bottom-right on the map, above NavigationControl.
 * Only renders when population mesh layer is toggled ON.
 */
export function YearSlider({ value, onChange, visible }: YearSliderProps) {
  const [localValue, setLocalValue] = useState(value);
  const timerRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  useEffect(() => {
    setLocalValue(value);
  }, [value]);

  useEffect(() => {
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, []);

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const newValue = Number(e.target.value);
      setLocalValue(newValue);

      if (timerRef.current) clearTimeout(timerRef.current);
      timerRef.current = setTimeout(() => {
        onChange(newValue);
      }, DEBOUNCE_MS);
    },
    [onChange],
  );

  if (!visible) return null;

  return (
    <fieldset
      className="absolute bottom-[100px] right-[10px] z-10 rounded px-3 py-2 border-none m-0 p-0"
      style={{
        background: "var(--bg-secondary)",
        border: "1px solid var(--border-primary)",
        minWidth: 160,
        padding: "8px 12px",
      }}
      aria-label="人口推計年度選択"
    >
      <div
        className="text-[9px] tracking-[0.1em] mb-1"
        style={{
          color: "var(--text-muted)",
          fontFamily: "var(--font-mono)",
        }}
      >
        人口推計年度
      </div>
      <div
        className="text-center text-sm font-bold mb-1"
        style={{
          color: "var(--layer-population)",
          fontFamily: "var(--font-mono)",
        }}
      >
        {localValue}
      </div>
      <input
        type="range"
        min={MIN_YEAR}
        max={MAX_YEAR}
        step={STEP}
        value={localValue}
        onChange={handleChange}
        className="w-full accent-[var(--layer-population)]"
        aria-label={`人口推計年度: ${localValue}年`}
        aria-valuemin={MIN_YEAR}
        aria-valuemax={MAX_YEAR}
        aria-valuenow={localValue}
      />
      <div
        className="flex justify-between text-[8px]"
        style={{
          color: "var(--text-muted)",
          fontFamily: "var(--font-mono)",
        }}
      >
        <span>{MIN_YEAR}</span>
        <span>{MAX_YEAR}</span>
      </div>
    </fieldset>
  );
}
