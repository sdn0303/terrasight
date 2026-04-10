"use client";

interface LayerToggleRowProps {
  id: string;
  label: string;
  /** Optional color swatch. CSS-variable strings like `var(--layer-flood)` are supported. */
  swatch?: string | undefined;
  checked: boolean;
  onToggle: (id: string) => void;
}

export function LayerToggleRow({
  id,
  label,
  swatch,
  checked,
  onToggle,
}: LayerToggleRowProps) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      onClick={() => onToggle(id)}
      className="flex w-full items-center gap-2 rounded-[10px] px-2 py-2 transition-colors hover:bg-[var(--neutral-50)]"
    >
      {swatch !== undefined && (
        <span
          data-testid="swatch"
          aria-hidden="true"
          style={{
            background: swatch,
            width: 10,
            height: 10,
            borderRadius: 3,
            flexShrink: 0,
          }}
        />
      )}
      <span
        className="flex-1 text-left text-[11px] font-medium"
        style={{ color: "var(--neutral-900)" }}
      >
        {label}
      </span>
      <span
        aria-hidden="true"
        className="relative inline-block h-4 w-7 rounded-full"
        style={{
          background: checked ? "var(--brand-indigo)" : "var(--neutral-200)",
          transition: "background 120ms ease",
        }}
      >
        <span
          className="absolute top-0.5 block h-3 w-3 rounded-full bg-white"
          style={{
            left: checked ? 14 : 2,
            transition: "left 120ms ease",
          }}
        />
      </span>
    </button>
  );
}
