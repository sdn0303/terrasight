export default function NotFound() {
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
        style={{ color: "var(--accent-warning)" }}
      >
        ── 404 NOT FOUND ──
      </div>
      <div className="text-sm" style={{ color: "var(--text-primary)" }}>
        This page does not exist.
      </div>
      <a
        href="/"
        className="px-4 py-2 rounded text-xs"
        style={{
          background: "var(--bg-tertiary)",
          color: "var(--accent-cyan)",
          border: "1px solid var(--border-primary)",
        }}
      >
        ← Return to map
      </a>
    </div>
  );
}
