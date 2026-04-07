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
        An unexpected error occurred.
        {error.digest && (
          <span
            className="block text-[10px] mt-1"
            style={{ color: "var(--text-muted)" }}
          >
            Reference: {error.digest}
          </span>
        )}
      </div>
      <button
        type="button"
        onClick={reset}
        className="px-4 py-2 rounded text-xs"
        style={{
          background: "var(--bg-tertiary)",
          color: "var(--accent-primary)",
          border: "1px solid var(--border-primary)",
        }}
      >
        RETRY
      </button>
      <a href="/" className="text-xs" style={{ color: "var(--text-muted)" }}>
        ← Return to map
      </a>
    </div>
  );
}
