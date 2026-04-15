import { clsx } from "clsx";
import type { LucideIcon } from "lucide-react";

interface SidebarItemProps {
  icon: LucideIcon;
  label: string;
  active: boolean;
  collapsed: boolean;
  onClick: () => void;
}

export function SidebarItem({
  icon: Icon,
  label,
  active,
  collapsed,
  onClick,
}: SidebarItemProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={clsx(
        "flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors duration-200",
        active ? "font-medium" : "hover:bg-[var(--panel-hover-bg)]",
        collapsed && "justify-center px-0",
      )}
      style={{
        color: active
          ? "var(--panel-text-primary)"
          : "var(--panel-text-secondary)",
        backgroundColor: active ? "var(--panel-active-bg)" : undefined,
      }}
      title={collapsed ? label : undefined}
    >
      <Icon className="h-5 w-5 shrink-0" />
      {!collapsed && <span className="truncate">{label}</span>}
    </button>
  );
}
