import { describe, expect, it } from "vitest";
import { isValidThemeId, THEMES } from "@/lib/theme-definitions";

describe("theme-definitions", () => {
  it("has 5 themes total", () => {
    expect(THEMES).toHaveLength(5);
  });

  it("all themes have required fields: id, label, icon, category", () => {
    for (const theme of THEMES) {
      expect(theme.id).toBeTruthy();
      expect(typeof theme.label).toBe("string");
      expect(theme.label.length).toBeGreaterThan(0);
      // Lucide icons are React components; they can be functions or objects (forwardRef)
      expect(theme.icon).toBeTruthy();
      expect(["explore", "view"]).toContain(theme.category);
    }
  });

  it("theme IDs are unique", () => {
    const ids = THEMES.map((t) => t.id);
    const unique = new Set(ids);
    expect(unique.size).toBe(ids.length);
  });

  it("explore category contains 'score'", () => {
    const exploreThemes = THEMES.filter((t) => t.category === "explore");
    const exploreIds = exploreThemes.map((t) => t.id);
    expect(exploreIds).toContain("score");
  });

  it("view category has 4 themes", () => {
    const viewThemes = THEMES.filter((t) => t.category === "view");
    expect(viewThemes).toHaveLength(4);
  });

  it("isValidThemeId returns true for valid IDs", () => {
    for (const theme of THEMES) {
      expect(isValidThemeId(theme.id)).toBe(true);
    }
  });

  it("isValidThemeId returns false for invalid IDs", () => {
    expect(isValidThemeId("nonexistent")).toBe(false);
    expect(isValidThemeId("")).toBe(false);
    expect(isValidThemeId("LAND-PRICE")).toBe(false);
  });
});
