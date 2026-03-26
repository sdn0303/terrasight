"use client";

interface ContextPanelProps {
  children: React.ReactNode;
}

export function ContextPanel({ children }: ContextPanelProps) {
  return (
    <aside
      className="fixed left-0 overflow-y-auto bg-neutral-900 border-r border-neutral-800"
      style={{
        top: 48,
        bottom: 28,
        width: 320,
        zIndex: 40,
      }}
      aria-label="Context panel"
    >
      {children}
    </aside>
  );
}
