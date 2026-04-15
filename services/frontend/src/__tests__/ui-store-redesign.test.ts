import { describe, it, expect, beforeEach } from "vitest";
import { useUIStore } from "@/stores/ui-store";

const INITIAL_STATE = {
  sidebarCollapsed: false,
  activeTheme: null,
  leftPanel: null,
  tableOpen: false,
  rightDrawer: null,
  selectedOpportunityId: null,
  baseMap: "light",
  locale: "ja",
} as const;

describe("ui-store state transitions", () => {
  beforeEach(() => {
    useUIStore.setState(INITIAL_STATE);
  });

  it("openTable() sets tableOpen and closes leftPanel", () => {
    useUIStore.setState({
      leftPanel: {
        type: "point-detail",
        data: { lat: 35.68, lng: 139.69 },
        activeTab: "land-price",
      },
    });
    useUIStore.getState().openTable();
    const s = useUIStore.getState();
    expect(s.tableOpen).toBe(true);
    expect(s.leftPanel).toBeNull();
  });

  it("openLeftPanel() closes table and rightDrawer", () => {
    useUIStore.setState({
      tableOpen: true,
      rightDrawer: { type: "opportunity", id: "opp-1" },
    });
    useUIStore.getState().openLeftPanel({ lat: 35.68, lng: 139.69 });
    const s = useUIStore.getState();
    expect(s.tableOpen).toBe(false);
    expect(s.rightDrawer).toBeNull();
    expect(s.leftPanel).not.toBeNull();
  });

  it("closeTable() clears rightDrawer and selectedOpportunityId", () => {
    useUIStore.setState({
      tableOpen: true,
      rightDrawer: { type: "opportunity", id: "opp-1" },
      selectedOpportunityId: "opp-1",
    });
    useUIStore.getState().closeTable();
    const s = useUIStore.getState();
    expect(s.tableOpen).toBe(false);
    expect(s.rightDrawer).toBeNull();
    expect(s.selectedOpportunityId).toBeNull();
  });

  it("setActiveTheme() clears leftPanel", () => {
    useUIStore.setState({
      leftPanel: {
        type: "point-detail",
        data: { lat: 35.68, lng: 139.69 },
        activeTab: "land-price",
      },
    });
    useUIStore.getState().setActiveTheme("hazard");
    const s = useUIStore.getState();
    expect(s.activeTheme).toBe("hazard");
    expect(s.leftPanel).toBeNull();
  });

  it("openOpportunityDrawer() sets selectedOpportunityId", () => {
    useUIStore.getState().openOpportunityDrawer("opp-42");
    const s = useUIStore.getState();
    expect(s.selectedOpportunityId).toBe("opp-42");
    expect(s.rightDrawer).toEqual({ type: "opportunity", id: "opp-42" });
  });

  it("leftPanel.activeTab defaults to activeTheme when set", () => {
    useUIStore.setState({ activeTheme: "hazard" });
    useUIStore.getState().openLeftPanel({ lat: 35.68, lng: 139.69 });
    const s = useUIStore.getState();
    expect(s.leftPanel?.activeTab).toBe("hazard");
  });

  it("leftPanel.activeTab defaults to 'land-price' when activeTheme is null", () => {
    useUIStore.setState({ activeTheme: null });
    useUIStore.getState().openLeftPanel({ lat: 35.68, lng: 139.69 });
    const s = useUIStore.getState();
    expect(s.leftPanel?.activeTab).toBe("land-price");
  });

  it("toggleSidebar() toggles sidebarCollapsed", () => {
    expect(useUIStore.getState().sidebarCollapsed).toBe(false);
    useUIStore.getState().toggleSidebar();
    expect(useUIStore.getState().sidebarCollapsed).toBe(true);
    useUIStore.getState().toggleSidebar();
    expect(useUIStore.getState().sidebarCollapsed).toBe(false);
  });

  it("closeRightDrawer() clears selectedOpportunityId", () => {
    useUIStore.setState({
      rightDrawer: { type: "opportunity", id: "opp-7" },
      selectedOpportunityId: "opp-7",
    });
    useUIStore.getState().closeRightDrawer();
    const s = useUIStore.getState();
    expect(s.rightDrawer).toBeNull();
    expect(s.selectedOpportunityId).toBeNull();
  });
});
