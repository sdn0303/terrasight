"use client";

interface ContextPanelProps {
  children: React.ReactNode;
}

export function ContextPanel({ children }: ContextPanelProps) {
  return (
    <aside
      className="fixed left-0 overflow-y-auto bg-ds-bg-secondary border-r border-ds-border-primary"
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
