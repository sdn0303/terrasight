import { create } from "zustand";
import { devtools } from "zustand/middleware";
import { DEFAULT_TAB, type TabId } from "@/features/tabs/tab-configs";

export type BaseMap = "dark" | "satellite";

export interface AreaSelection {
  code: string;
  name: string;
  level: "prefecture" | "municipality";
  lat: number;
  lng: number;
}

interface UIState {
  // Active tab (10-tab exclusive switch)
  activeTab: TabId;
  setActiveTab: (tab: TabId) => void;

  // Detail panel (badge/point click)
  selectedArea: AreaSelection | null;
  setSelectedArea: (area: AreaSelection | null) => void;

  // Sidebar (icon-only, no expand/collapse in v3)
  sidebarSection: "explore" | "settings" | null;
  setSidebarSection: (s: "explore" | "settings" | null) => void;

  // Infrastructure toggles (sidebar ON/OFF)
  showSchools: boolean;
  showMedical: boolean;
  toggleSchools: () => void;
  toggleMedical: () => void;

  // Legend
  legendCollapsed: boolean;
  toggleLegend: () => void;

  // Map style
  baseMap: BaseMap;
  setBaseMap: (m: BaseMap) => void;

  // Locale
  locale: "ja" | "en";
  setLocale: (l: "ja" | "en") => void;
}

export const useUIStore = create<UIState>()(
  devtools(
    (set) => ({
      // Tab (default: overview)
      activeTab: DEFAULT_TAB,
      setActiveTab: (tab) => set({ activeTab: tab, selectedArea: null }),

      // Detail panel
      selectedArea: null,
      setSelectedArea: (area) => set({ selectedArea: area }),

      // Sidebar
      sidebarSection: null,
      setSidebarSection: (s) => set({ sidebarSection: s }),

      // Infrastructure
      showSchools: false,
      showMedical: false,
      toggleSchools: () => set((s) => ({ showSchools: !s.showSchools })),
      toggleMedical: () => set((s) => ({ showMedical: !s.showMedical })),

      // Legend
      legendCollapsed: false,
      toggleLegend: () => set((s) => ({ legendCollapsed: !s.legendCollapsed })),

      // Map style (v3: dark default, no light)
      baseMap: "dark",
      setBaseMap: (m) => set({ baseMap: m }),

      // Locale
      locale: "ja",
      setLocale: (l) => set({ locale: l }),
    }),
    { name: "ui-store" },
  ),
);
