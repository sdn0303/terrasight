

import {
  type CSSProperties,
  type KeyboardEvent as ReactKeyboardEvent,
  type ReactNode,
  useRef,
} from "react";

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
 *
 * Implements the WAI-ARIA radiogroup pattern:
 *   - Only the currently-selected option is tabbable (roving tabindex)
 *   - Arrow keys (and Home/End) move selection and focus within the group
 *   - Space/Enter are native on `<button>` and select via onClick
 */
export function SegmentedToggle<T extends string>({
  options,
  value,
  onChange,
  "aria-label": ariaLabel,
}: SegmentedToggleProps<T>) {
  const buttonRefs = useRef<(HTMLButtonElement | null)[]>([]);
  const activeIndex = options.findIndex((o) => o.value === value);

  const moveTo = (nextIndex: number) => {
    const count = options.length;
    if (count === 0) return;
    const wrapped = ((nextIndex % count) + count) % count;
    const next = options[wrapped];
    if (!next) return;
    onChange(next.value);
    buttonRefs.current[wrapped]?.focus();
  };

  const handleKeyDown = (
    e: ReactKeyboardEvent<HTMLButtonElement>,
    index: number,
  ) => {
    switch (e.key) {
      case "ArrowRight":
      case "ArrowDown":
        e.preventDefault();
        moveTo(index + 1);
        break;
      case "ArrowLeft":
      case "ArrowUp":
        e.preventDefault();
        moveTo(index - 1);
        break;
      case "Home":
        e.preventDefault();
        moveTo(0);
        break;
      case "End":
        e.preventDefault();
        moveTo(options.length - 1);
        break;
      default:
        break;
    }
  };

  return (
    <div role="radiogroup" aria-label={ariaLabel} className="flex gap-1.5">
      {options.map((opt, index) => {
        const isActive = opt.value === value;
        // When no option matches `value`, fall back to making the first one
        // tabbable so keyboard users still have an entry point.
        const isTabTarget =
          activeIndex >= 0 ? index === activeIndex : index === 0;
        return (
          // biome-ignore lint/a11y/useSemanticElements: styled segmented control intentionally uses button+role="radio" for custom appearance
          <button
            key={opt.value}
            ref={(el) => {
              buttonRefs.current[index] = el;
            }}
            type="button"
            role="radio"
            aria-checked={isActive}
            tabIndex={isTabTarget ? 0 : -1}
            onClick={() => onChange(opt.value)}
            onKeyDown={(e) => handleKeyDown(e, index)}
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
