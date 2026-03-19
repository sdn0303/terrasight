export function CRTOverlay() {
  return (
    <>
      {/* Vignette layer */}
      <div
        aria-hidden="true"
        className="pointer-events-none fixed inset-0"
        style={{
          zIndex: 200,
          background:
            "radial-gradient(circle, transparent 40%, rgba(0,0,0,0.8) 100%)",
        }}
      />
      {/* Scanlines layer */}
      <div
        aria-hidden="true"
        className="pointer-events-none fixed inset-0"
        style={{
          zIndex: 300,
          opacity: 0.05,
          background:
            "linear-gradient(rgba(255,255,255,0.1) 1px, transparent 1px)",
          backgroundSize: "100% 4px",
        }}
      />
    </>
  );
}
