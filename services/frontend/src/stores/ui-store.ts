import { create } from "zustand";
import { devtools } from "zustand/middleware";
import type { ThemeId } from "@/lib/theme-definitions";

export type BaseMap = "light" | "dark" | "satellite";

export interface PointDetailData {
	lat: number;
	lng: number;
	address?: string;
	featureProperties?: Record<string, unknown>;
}

interface LeftPanel {
	type: "point-detail";
	data: PointDetailData;
	activeTab: ThemeId;
}

type RightDrawer =
	| { type: "opportunity"; id: string }
	| { type: "map-point"; data: PointDetailData; activeTab: ThemeId };

interface UIState {
	// Sidebar
	sidebarCollapsed: boolean;
	toggleSidebar: () => void;

	// Active theme (exclusive)
	activeTheme: ThemeId | null;
	setActiveTheme: (t: ThemeId | null) => void;

	// Left panel (map-only point detail)
	leftPanel: LeftPanel | null;
	openLeftPanel: (data: PointDetailData) => void;
	closeLeftPanel: () => void;
	setLeftPanelTab: (tab: ThemeId) => void;

	// Opportunities table
	tableOpen: boolean;
	openTable: () => void;
	closeTable: () => void;

	// Right drawer (table mode only)
	rightDrawer: RightDrawer | null;
	openOpportunityDrawer: (id: string) => void;
	openMapPointDrawer: (data: PointDetailData) => void;
	closeRightDrawer: () => void;

	// Selected opportunity (table highlight + drawer link)
	selectedOpportunityId: string | null;
	setSelectedOpportunityId: (id: string | null) => void;

	// Map style
	baseMap: BaseMap;
	setBaseMap: (m: BaseMap) => void;

	// Locale
	locale: "ja" | "en";
	setLocale: (l: "ja" | "en") => void;
}

export const useUIStore = create<UIState>()(
	devtools(
		(set, _get) => ({
			// Sidebar
			sidebarCollapsed: false,
			toggleSidebar: () =>
				set((s) => ({ sidebarCollapsed: !s.sidebarCollapsed })),

			// Theme (exclusive)
			activeTheme: null,
			setActiveTheme: (t) => set({ activeTheme: t, leftPanel: null }),

			// Left panel
			leftPanel: null,
			openLeftPanel: (data) =>
				set((s) => ({
					leftPanel: {
						type: "point-detail",
						data,
						activeTab: s.activeTheme ?? "land-price",
					},
					tableOpen: false,
					rightDrawer: null,
				})),
			closeLeftPanel: () => set({ leftPanel: null }),
			setLeftPanelTab: (tab) =>
				set((s) =>
					s.leftPanel
						? { leftPanel: { ...s.leftPanel, activeTab: tab } }
						: {},
				),

			// Table
			tableOpen: false,
			openTable: () => set({ tableOpen: true, leftPanel: null }),
			closeTable: () =>
				set({
					tableOpen: false,
					rightDrawer: null,
					selectedOpportunityId: null,
				}),

			// Right drawer
			rightDrawer: null,
			openOpportunityDrawer: (id) =>
				set({
					rightDrawer: { type: "opportunity", id },
					selectedOpportunityId: id,
				}),
			openMapPointDrawer: (data) =>
				set((s) => ({
					rightDrawer: {
						type: "map-point",
						data,
						activeTab: s.activeTheme ?? "land-price",
					},
				})),
			closeRightDrawer: () =>
				set({ rightDrawer: null, selectedOpportunityId: null }),

			// Selected opportunity
			selectedOpportunityId: null,
			setSelectedOpportunityId: (id) => set({ selectedOpportunityId: id }),

			// Map style
			baseMap: "light",
			setBaseMap: (m) => set({ baseMap: m }),

			// Locale
			locale: "ja",
			setLocale: (l) => set({ locale: l }),
		}),
		{ name: "ui-store" },
	),
);
