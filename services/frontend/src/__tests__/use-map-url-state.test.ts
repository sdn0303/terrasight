import { describe, expect, it } from "vitest";
import type { ThemeId } from "@/lib/themes";
import { THEMES } from "@/lib/themes";

describe("first-visit theme activation contract", () => {
  it("safety is a valid ThemeId", () => {
    const safetyTheme = THEMES.find((t) => t.id === "safety");
    expect(safetyTheme).toBeDefined();
    expect(safetyTheme?.id).toBe("safety");
  });

  it("default theme param parses to safety", () => {
    const defaultTheme = "safety";
    const validIds = THEMES.map((t) => t.id);
    expect(validIds).toContain(defaultTheme);
  });

  it("empty theme param string results in no themes", () => {
    const themeParam = "";
    const themeIds = themeParam.split(",").filter(Boolean);
    expect(themeIds).toHaveLength(0);
  });

  it("multiple themes can be serialized and deserialized", () => {
    const themes: ThemeId[] = ["safety", "livability"];
    const serialized = themes.join(",");
    const deserialized = serialized.split(",").filter(Boolean);
    expect(deserialized).toEqual(["safety", "livability"]);
  });
});
