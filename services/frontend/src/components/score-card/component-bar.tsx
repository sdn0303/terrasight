interface ComponentBarProps {
  label: string;
  value: number;
  max: number;
}

export function ComponentBar({ label, value, max }: ComponentBarProps) {
  const pct = max > 0 ? (value / max) * 100 : 0;
  return (
    <div
      className="flex items-center gap-2 text-[10px]"
      style={{ fontFamily: "var(--font-mono)" }}
    >
      <span className="w-16 text-right" style={{ color: "var(--text-muted)" }}>
        {label}:
      </span>
      <div
        className="flex-1 h-1.5 rounded"
        style={{ background: "var(--bg-tertiary)" }}
        role="progressbar"
        aria-valuenow={value}
        aria-valuemin={0}
        aria-valuemax={max}
        aria-label={label}
      >
        <div
          className="h-full rounded"
          style={{ width: `${pct}%`, background: "var(--accent-cyan)" }}
        />
      </div>
      <span style={{ color: "var(--text-secondary)" }}>
        {value}/{max}
      </span>
    </div>
  );
}
