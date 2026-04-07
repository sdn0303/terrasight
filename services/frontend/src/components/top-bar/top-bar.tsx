"use client";

import { Search, Settings } from "lucide-react";
import { useTranslations } from "next-intl";
import { LocaleToggle } from "./locale-toggle";
import { ModeTabs } from "./mode-tabs";

export function TopBar() {
  const t = useTranslations();

  return (
    <header
      className="fixed top-0 left-0 right-0 z-50 flex items-center justify-between px-4 gap-4"
      style={{
        height: "56px",
        background: "rgba(17, 24, 39, 0.80)",
        backdropFilter: "blur(12px)",
        borderBottom: "1px solid var(--border-primary)",
      }}
    >
      {/* Left: Brand + Mode Tabs */}
      <div className="flex items-center gap-4">
        <span
          className="text-sm font-semibold tracking-tight"
          style={{ color: "var(--accent-primary)" }}
        >
          Terrasight
        </span>
        <ModeTabs />
      </div>

      {/* Center: Search */}
      <div className="flex-1 max-w-md mx-4">
        <div
          className="flex items-center gap-2 rounded-full px-4 py-1.5 text-sm transition-colors"
          style={{
            background: "var(--surface-elevated)",
            border: "1px solid var(--border-primary)",
            color: "var(--text-muted)",
          }}
        >
          <Search size={14} style={{ color: "var(--text-muted)" }} />
          <span className="text-xs">{t("search.placeholder")}</span>
        </div>
      </div>

      {/* Right: Controls */}
      <div className="flex items-center gap-2">
        <LocaleToggle />
        <button
          type="button"
          className="p-2 rounded-lg transition-colors cursor-pointer hover:bg-ds-hover-accent"
          aria-label="Settings"
        >
          <Settings size={16} style={{ color: "var(--text-secondary)" }} />
        </button>
        <div
          className="w-7 h-7 rounded-full flex items-center justify-center text-[10px] font-semibold"
          style={{
            background: "var(--accent-primary)",
            color: "var(--bg-primary)",
          }}
        >
          T
        </div>
      </div>
    </header>
  );
}
