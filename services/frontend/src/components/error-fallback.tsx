"use client";

interface ErrorFallbackProps {
  error: Error & { digest?: string };
  reset: () => void;
}

export function ErrorFallback({ error, reset }: ErrorFallbackProps) {
  return (
    <div
      className="flex flex-col items-center justify-center h-screen gap-4"
      style={{
        background: "var(--bg-primary)",
        fontFamily: "var(--font-mono)",
      }}
    >
      <div
        className="text-[9px] tracking-[0.15em]"
        style={{ color: "var(--accent-danger)" }}
      >
        ── SYSTEM ERROR ──
      </div>
      <div className="text-sm" style={{ color: "var(--text-primary)" }}>
        {error.message}
      </div>
      <button
        type="button"
        onClick={reset}
        className="px-4 py-2 rounded text-xs"
        style={{
          background: "var(--bg-tertiary)",
          color: "var(--accent-cyan)",
          border: "1px solid var(--border-primary)",
        }}
      >
        RETRY
      </button>
      <a
        href="/"
        className="text-xs"
        style={{ color: "var(--text-muted)" }}
      >
        ← Return to map
      </a>
    </div>
  );
}
