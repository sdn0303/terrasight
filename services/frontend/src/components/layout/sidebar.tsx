import { ChevronLeft, ChevronRight, Search, Settings } from "lucide-react";
import { useUIStore } from "@/stores/ui-store";
import { THEMES, type ThemeId } from "@/lib/theme-definitions";
import { SidebarItem } from "./sidebar-item";

const SIDEBAR_EXPANDED_W = 200;
const SIDEBAR_COLLAPSED_W = 56;

export function Sidebar() {
  const collapsed = useUIStore((s) => s.sidebarCollapsed);
  const toggle = useUIStore((s) => s.toggleSidebar);
  const activeTheme = useUIStore((s) => s.activeTheme);
  const setActiveTheme = useUIStore((s) => s.setActiveTheme);
  const tableOpen = useUIStore((s) => s.tableOpen);
  const openTable = useUIStore((s) => s.openTable);
  const closeTable = useUIStore((s) => s.closeTable);

  const width = collapsed ? SIDEBAR_COLLAPSED_W : SIDEBAR_EXPANDED_W;
  const explore = THEMES.filter((t) => t.category === "explore");
  const view = THEMES.filter((t) => t.category === "view");

  const handleThemeClick = (id: ThemeId) => {
    setActiveTheme(activeTheme === id ? null : id);
  };

  const handleOpportunitiesClick = () => {
    if (tableOpen) {
      closeTable();
    } else {
      openTable();
    }
  };

  return (
    <nav
      className="absolute left-0 top-0 z-30 flex h-full flex-col overflow-hidden border-r"
      style={{
        width,
        transition: "width 0.3s ease",
        backgroundColor: "var(--panel-bg)",
        borderColor: "var(--panel-border)",
      }}
    >
      {/* Logo + collapse toggle */}
      <div className="flex h-14 items-center justify-between px-3">
        {!collapsed && (
          <span
            className="text-sm font-bold tracking-tight"
            style={{ color: "var(--panel-text-primary)" }}
          >
            Terrasight
          </span>
        )}
        <button
          type="button"
          onClick={toggle}
          className="rounded-md p-1 transition-colors hover:bg-[var(--panel-hover-bg)]"
          style={{ color: "var(--panel-text-secondary)" }}
          aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
        >
          {collapsed ? (
            <ChevronRight className="h-4 w-4" />
          ) : (
            <ChevronLeft className="h-4 w-4" />
          )}
        </button>
      </div>

      {/* Section: explore */}
      {!collapsed && (
        <div className="px-3 pb-1 pt-3">
          <span
            className="text-[10px] font-semibold uppercase tracking-widest"
            style={{ color: "var(--panel-text-secondary)" }}
          >
            探す
          </span>
        </div>
      )}

      <div className="flex flex-col gap-0.5 px-2">
        <SidebarItem
          icon={Search}
          label="Opportunities"
          active={tableOpen}
          collapsed={collapsed}
          onClick={handleOpportunitiesClick}
        />
        {explore.map((t) => (
          <SidebarItem
            key={t.id}
            icon={t.icon}
            label={t.label}
            active={activeTheme === t.id}
            collapsed={collapsed}
            onClick={() => handleThemeClick(t.id)}
          />
        ))}
      </div>

      {/* Section: view */}
      {!collapsed && (
        <div className="px-3 pb-1 pt-4">
          <span
            className="text-[10px] font-semibold uppercase tracking-widest"
            style={{ color: "var(--panel-text-secondary)" }}
          >
            見る
          </span>
        </div>
      )}

      <div className="flex flex-col gap-0.5 px-2">
        {view.map((t) => (
          <SidebarItem
            key={t.id}
            icon={t.icon}
            label={t.label}
            active={activeTheme === t.id}
            collapsed={collapsed}
            onClick={() => handleThemeClick(t.id)}
          />
        ))}
      </div>

      {/* Spacer */}
      <div className="flex-1" />

      {/* Section: settings */}
      {!collapsed && (
        <div className="px-3 pb-1">
          <span
            className="text-[10px] font-semibold uppercase tracking-widest"
            style={{ color: "var(--panel-text-secondary)" }}
          >
            設定
          </span>
        </div>
      )}

      <div className="flex flex-col gap-0.5 px-2 pb-4">
        <SidebarItem
          icon={Settings}
          label="設定"
          active={false}
          collapsed={collapsed}
          onClick={() => {}}
        />
      </div>
    </nav>
  );
}
