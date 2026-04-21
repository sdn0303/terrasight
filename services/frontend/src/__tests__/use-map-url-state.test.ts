import { describe, expect, it } from "vitest";
import { isValidCoordinate } from "@/hooks/use-map-url-state";
import { isValidThemeId, THEMES } from "@/lib/theme-definitions";

describe("isValidCoordinate", () => {
  it("accepts Tokyo Station coordinates", () => {
    expect(isValidCoordinate(35.681, 139.767)).toBe(true);
  });

  it("accepts equator / zero coordinates", () => {
    expect(isValidCoordinate(0, 0)).toBe(true);
  });

  it("accepts extreme but valid boundaries", () => {
    expect(isValidCoordinate(90, 180)).toBe(true);
    expect(isValidCoordinate(-90, -180)).toBe(true);
  });

  it("rejects null coordinates", () => {
    expect(isValidCoordinate(null, 139.767)).toBe(false);
    expect(isValidCoordinate(35.681, null)).toBe(false);
    expect(isValidCoordinate(null, null)).toBe(false);
  });

  it("rejects NaN", () => {
    expect(isValidCoordinate(Number.NaN, 139.767)).toBe(false);
    expect(isValidCoordinate(35.681, Number.NaN)).toBe(false);
  });

  it("rejects Infinity", () => {
    expect(isValidCoordinate(Number.POSITIVE_INFINITY, 0)).toBe(false);
    expect(isValidCoordinate(0, Number.NEGATIVE_INFINITY)).toBe(false);
  });

  it("rejects out-of-range latitude", () => {
    expect(isValidCoordinate(91, 0)).toBe(false);
    expect(isValidCoordinate(-91, 0)).toBe(false);
  });

  it("rejects out-of-range longitude", () => {
    expect(isValidCoordinate(0, 181)).toBe(false);
    expect(isValidCoordinate(0, -181)).toBe(false);
  });
});

describe("isValidThemeId", () => {
  it("accepts all defined ThemeId values", () => {
    for (const theme of THEMES) {
      expect(isValidThemeId(theme.id)).toBe(true);
    }
  });

  it("accepts each TabId individually", () => {
    expect(isValidThemeId("overview")).toBe(true);
    expect(isValidThemeId("land-price")).toBe(true);
    expect(isValidThemeId("hazard")).toBe(true);
    expect(isValidThemeId("transactions")).toBe(true);
    expect(isValidThemeId("stations")).toBe(true);
    expect(isValidThemeId("population")).toBe(true);
    expect(isValidThemeId("vacancy")).toBe(true);
    expect(isValidThemeId("yield")).toBe(true);
    expect(isValidThemeId("ground")).toBe(true);
    expect(isValidThemeId("zoning")).toBe(true);
  });

  it("rejects legacy / unknown theme IDs", () => {
    expect(isValidThemeId("safety")).toBe(false);
    expect(isValidThemeId("livability")).toBe(false);
    expect(isValidThemeId("station")).toBe(false);
    expect(isValidThemeId("score")).toBe(false);
    expect(isValidThemeId("")).toBe(false);
    expect(isValidThemeId("unknown")).toBe(false);
  });
});

describe("URL theme param contract", () => {
  it("empty theme param results in no theme to restore", () => {
    const themeParam = "";
    expect(isValidThemeId(themeParam)).toBe(false);
  });

  it("valid theme param passes the guard", () => {
    expect(isValidThemeId("land-price")).toBe(true);
    expect(isValidThemeId("overview")).toBe(true);
  });
});
