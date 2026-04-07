"use client";

interface ContextPanelProps {
  children: React.ReactNode;
}

export function ContextPanel({ children }: ContextPanelProps) {
  return (
    <aside
      className="fixed left-0 overflow-y-auto"
      style={{
        top: 56,
        bottom: 28,
        width: 320,
        zIndex: 40,
        background: "var(--bg-sidebar)",
      }}
      aria-label="Context panel"
    >
      <div
        className="mx-2 my-2 rounded-xl h-[calc(100%-16px)] overflow-y-auto"
        style={{
          background: "rgba(10, 10, 18, 0.4)",
          backdropFilter: "blur(8px)",
          border: "1px solid rgba(99, 102, 241, 0.08)",
        }}
      >
        {children}
      </div>
    </aside>
  );
}
