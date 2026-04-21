import { useUIStore } from "@/stores/ui-store";
import { TABS, type TabId } from "./tab-configs";

export function FloatingTabBar() {
  const activeTab = useUIStore((s) => s.activeTab);
  const setActiveTab = useUIStore((s) => s.setActiveTab);

  return (
    <div
      className="fixed flex items-center gap-0.5 px-1"
      style={{
        left: "calc(var(--ts-sidebar-width) + var(--ts-gap-panel) * 2)",
        top: "var(--ts-gap-panel)",
        height: "var(--ts-tab-height)",
        background: "var(--ts-bg-sidebar)",
        borderRadius: "var(--ts-panel-radius)",
        zIndex: 10,
        overflowX: "auto",
        maxWidth:
          "calc(100vw - var(--ts-sidebar-width) - var(--ts-gap-panel) * 3 - 220px)",
      }}
      role="tablist"
      aria-label="Category tabs"
    >
      {TABS.map((tab) => (
        <TabItem
          key={tab.id}
          id={tab.id}
          label={tab.label}
          Icon={tab.icon}
          isActive={activeTab === tab.id}
          onClick={() => setActiveTab(tab.id)}
        />
      ))}
    </div>
  );
}

function TabItem({
  id,
  label,
  Icon,
  isActive,
  onClick,
}: {
  id: TabId;
  label: string;
  Icon: React.ElementType;
  isActive: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      role="tab"
      id={`tab-${id}`}
      aria-selected={isActive}
      onClick={onClick}
      className="flex items-center gap-1.5 px-3 h-9 rounded-lg whitespace-nowrap transition-colors text-xs font-medium"
      style={{
        background: isActive ? "var(--ts-bg-tab-active)" : "transparent",
        color: isActive ? "var(--ts-accent)" : "var(--ts-text-muted)",
      }}
    >
      <Icon size={14} />
      {label}
    </button>
  );
}
