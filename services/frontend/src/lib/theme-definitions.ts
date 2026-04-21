/**
 * Backwards-compatibility shim for hooks that still reference ThemeId.
 * New code should import from @/features/tabs/tab-configs instead.
 */
import { isValidTabId, TABS, type TabId } from "@/features/tabs/tab-configs";

export type ThemeId = TabId;
export type SidebarCategory = "explore" | "view";

export interface ThemeDefinition {
  id: ThemeId;
  label: string;
  icon: React.ElementType;
  category: SidebarCategory;
}

export const THEMES: ThemeDefinition[] = TABS.map((t) => ({
  ...t,
  category: "view" as SidebarCategory,
}));

export { isValidTabId as isValidThemeId };
