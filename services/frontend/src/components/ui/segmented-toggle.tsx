"use client";

import type { CSSProperties, ReactNode } from "react";

interface SegmentedToggleOption<T extends string> {
  value: T;
  label: ReactNode;
  activeStyle?: CSSProperties;
}

interface SegmentedToggleProps<T extends string> {
  options: SegmentedToggleOption<T>[];
  value: T;
  onChange: (v: T) => void;
  "aria-label"?: string;
}

/**
 * Reusable segmented toggle used for RiskMax (Low/Mid/High), BaseMap (Light/Dark/Satellite),
 * and other discrete choice controls.
 */
export function SegmentedToggle<T extends string>({
  options,
  value,
  onChange,
  "aria-label": ariaLabel,
}: SegmentedToggleProps<T>) {
  return (
    <div role="radiogroup" aria-label={ariaLabel} className="flex gap-1.5">
      {options.map((opt) => {
        const isActive = opt.value === value;
        return (
          // biome-ignore lint/a11y/useSemanticElements: styled segmented control intentionally uses button+role="radio" for custom appearance
          <button
            key={opt.value}
            type="button"
            role="radio"
            aria-checked={isActive}
            onClick={() => onChange(opt.value)}
            className="flex-1 rounded-lg py-1.5 text-center text-[10px] font-extrabold transition-colors"
            style={
              isActive
                ? (opt.activeStyle ?? {
                    background: "var(--card-fill-solid)",
                    color: "var(--neutral-900)",
                    border: "2px solid var(--brand-indigo)",
                  })
                : {
                    background: "var(--neutral-50)",
                    color: "var(--neutral-400)",
                    border: "1px solid var(--neutral-100)",
                  }
            }
          >
            {opt.label}
          </button>
        );
      })}
    </div>
  );
}
