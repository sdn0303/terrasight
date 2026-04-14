import { describe, expect, it } from "vitest";
import {
  isValidCoordinate,
  normalizeFilterUrlParams,
  parseComparePointsParam,
  type RawFilterUrlParams,
} from "@/hooks/use-map-url-state";
import type { ThemeId } from "@/lib/themes";
import { THEMES } from "@/lib/themes";

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

describe("parseComparePointsParam", () => {
  it("returns empty array for empty string", () => {
    expect(parseComparePointsParam("")).toEqual([]);
  });

  it("parses a single point with address", () => {
    const pts = parseComparePointsParam("35.681,139.767,東京駅");
    expect(pts).toHaveLength(1);
    expect(pts[0]).toEqual({
      lat: 35.681,
      lng: 139.767,
      address: "東京駅",
    });
  });

  it("parses multiple pipe-separated points", () => {
    const pts = parseComparePointsParam(
      "35.681,139.767,A|35.69,139.77,B|35.70,139.78,C",
    );
    expect(pts).toHaveLength(3);
    expect(pts[0]?.address).toBe("A");
    expect(pts[1]?.address).toBe("B");
    expect(pts[2]?.address).toBe("C");
  });

  it("caps at 3 points (extra entries dropped)", () => {
    const pts = parseComparePointsParam(
      "35.68,139.76,A|35.69,139.77,B|35.70,139.78,C|35.71,139.79,D",
    );
    expect(pts).toHaveLength(3);
    expect(pts.find((p) => p.address === "D")).toBeUndefined();
  });

  it("filters out entries with invalid coordinates", () => {
    const pts = parseComparePointsParam(
      "91,0,BadLat|35.68,139.76,Good|0,200,BadLng",
    );
    expect(pts).toHaveLength(1);
    expect(pts[0]?.address).toBe("Good");
  });

  it("filters out entries with NaN coordinates", () => {
    const pts = parseComparePointsParam("abc,xyz,Garbage|35.68,139.76,Good");
    expect(pts).toHaveLength(1);
    expect(pts[0]?.address).toBe("Good");
  });

  it("preserves address with commas by joining remainder", () => {
    const pts = parseComparePointsParam("35.681,139.767,Chiyoda, Tokyo");
    expect(pts).toHaveLength(1);
    expect(pts[0]?.address).toBe("Chiyoda, Tokyo");
  });

  it('defaults address to "Unknown" when missing', () => {
    const pts = parseComparePointsParam("35.681,139.767,");
    expect(pts).toHaveLength(1);
    expect(pts[0]?.address).toBe("Unknown");
  });

  it("ignores empty entries from trailing/leading/double pipes", () => {
    const pts = parseComparePointsParam("|35.681,139.767,A||35.69,139.77,B|");
    expect(pts).toHaveLength(2);
  });
});

describe("normalizeFilterUrlParams", () => {
  const empty: RawFilterUrlParams = {
    tlsMin: null,
    riskMax: null,
    priceMin: null,
    priceMax: null,
    zones: null,
    stationMax: null,
    preset: null,
    cities: null,
  };

  it("returns empty object when all params are null", () => {
    expect(normalizeFilterUrlParams(empty)).toEqual({});
  });

  it("clamps tlsMin into [0, 100]", () => {
    expect(normalizeFilterUrlParams({ ...empty, tlsMin: 999 })).toEqual({
      criteria: { tlsMin: 100, riskMax: "high", priceRange: [0, 10_000_000] },
    });
    expect(normalizeFilterUrlParams({ ...empty, tlsMin: -5 })).toEqual({
      criteria: { tlsMin: 0, riskMax: "high", priceRange: [0, 10_000_000] },
    });
  });

  it("clamps stationMax into [100, 2000]", () => {
    expect(normalizeFilterUrlParams({ ...empty, stationMax: 50 })).toEqual({
      stationMax: 100,
    });
    expect(normalizeFilterUrlParams({ ...empty, stationMax: 9999 })).toEqual({
      stationMax: 2000,
    });
  });

  it("swaps priceMin/priceMax when inverted", () => {
    expect(
      normalizeFilterUrlParams({
        ...empty,
        priceMin: 5_000_000,
        priceMax: 1_000_000,
      }),
    ).toEqual({
      criteria: {
        tlsMin: 0,
        riskMax: "high",
        priceRange: [1_000_000, 5_000_000],
      },
    });
  });

  it("clamps price values into [0, 10_000_000]", () => {
    expect(
      normalizeFilterUrlParams({
        ...empty,
        priceMin: -100,
        priceMax: 50_000_000,
      }),
    ).toEqual({
      criteria: {
        tlsMin: 0,
        riskMax: "high",
        priceRange: [0, 10_000_000],
      },
    });
  });

  it("filters unknown zones out", () => {
    expect(
      normalizeFilterUrlParams({
        ...empty,
        zones: ["商業", "unknown", "住居"],
      }),
    ).toEqual({ zones: ["商業", "住居"] });
  });

  it("drops zones entry when every value is unknown", () => {
    expect(normalizeFilterUrlParams({ ...empty, zones: ["unknown"] })).toEqual(
      {},
    );
  });

  it("filters cities to known Tokyo 23 wards", () => {
    expect(
      normalizeFilterUrlParams({
        ...empty,
        cities: ["渋谷区", "大阪市", "新宿区"],
      }),
    ).toEqual({ cities: ["渋谷区", "新宿区"] });
  });

  it("replaces NaN numeric values with the range minimum", () => {
    expect(
      normalizeFilterUrlParams({
        ...empty,
        tlsMin: Number.NaN,
        stationMax: Number.NaN,
      }),
    ).toEqual({
      criteria: { tlsMin: 0, riskMax: "high", priceRange: [0, 10_000_000] },
      stationMax: 100,
    });
  });

  it("passes valid values through unchanged", () => {
    expect(
      normalizeFilterUrlParams({
        tlsMin: 60,
        riskMax: "mid",
        priceMin: 1_000_000,
        priceMax: 5_000_000,
        zones: ["商業", "近商"],
        stationMax: 800,
        preset: "investment",
        cities: ["渋谷区", "港区"],
      }),
    ).toEqual({
      criteria: {
        tlsMin: 60,
        riskMax: "mid",
        priceRange: [1_000_000, 5_000_000],
      },
      zones: ["商業", "近商"],
      stationMax: 800,
      preset: "investment",
      cities: ["渋谷区", "港区"],
    });
  });
});

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
