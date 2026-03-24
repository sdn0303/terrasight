interface TruncationInfo {
  layer: string;
  count: number;
  limit: number;
}

interface StatusBarProps {
  lat: number;
  lng: number;
  zoom: number;
  isLoading: boolean;
  isDemoMode: boolean;
  truncatedLayers?: TruncationInfo[];
}

export function StatusBar({
  lat,
  lng,
  zoom,
  isLoading,
  isDemoMode,
  truncatedLayers,
}: StatusBarProps) {
  return (
    <div
      role="status"
      aria-live="polite"
      aria-label="Map status"
      className="fixed bottom-0 left-0 right-0 flex items-center gap-4 px-4 overflow-x-auto"
      style={{
        height: "28px",
        fontSize: "10px",
        fontFamily: "var(--font-mono)",
        background: "var(--bg-primary)",
        borderTop: "1px solid var(--border-primary)",
        color: "var(--text-muted)",
        zIndex: 20,
      }}
    >
      <span>
        {lat.toFixed(4)}°N {lng.toFixed(4)}°E
      </span>
      <span>Z:{zoom.toFixed(1)}</span>
      {isDemoMode && (
        <span style={{ color: "var(--accent-warning)" }}>● DEMO</span>
      )}
      {isLoading && (
        <span style={{ color: "var(--accent-cyan)" }}>◌ LOADING...</span>
      )}
      {truncatedLayers && truncatedLayers.length > 0 && (
        <span
          style={{ color: "var(--accent-warning)" }}
          aria-label="Data truncation warning"
        >
          {truncatedLayers
            .map((t) => `⚠ ${t.layer}: ${t.count}/${t.limit} ▸ ズームインで全件表示`)
            .join(" · ")}
        </span>
      )}
    </div>
  );
}
