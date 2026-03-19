export default function CRTOverlay() {
  return (
    <>
      {/* Vignette */}
      <div
        className="fixed inset-0 pointer-events-none z-[2]"
        style={{
          background:
            "radial-gradient(circle, transparent 40%, rgba(0,0,0,0.7) 100%)",
        }}
      />
      {/* Scanlines */}
      <div
        className="fixed inset-0 pointer-events-none z-[3] opacity-[0.03]"
        style={{
          backgroundImage:
            "linear-gradient(rgba(255,255,255,0.1) 1px, transparent 1px)",
          backgroundSize: "100% 4px",
        }}
      />
    </>
  );
}
