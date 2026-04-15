import { describe, expect, it } from "vitest";
import {
  BRAND,
  LAYER,
  NEUTRAL,
  OVERLAY,
  PRICE_STOPS,
  SCORE,
  SEMANTIC,
  TREND,
} from "@/lib/palette";

const HEX_RE = /^#[0-9a-fA-F]{6}$/;
const RGBA_RE = /^rgba\(\d+,\s*\d+,\s*\d+,\s*[\d.]+\)$/;

describe("palette", () => {
  it("all BRAND values are valid hex", () => {
    for (const [key, value] of Object.entries(BRAND)) {
      expect(value, `BRAND.${key}`).toMatch(HEX_RE);
    }
  });

  it("all NEUTRAL values are valid hex", () => {
    for (const [key, value] of Object.entries(NEUTRAL)) {
      expect(value, `NEUTRAL.${key}`).toMatch(HEX_RE);
    }
  });

  it("all SEMANTIC values are valid hex", () => {
    for (const [key, value] of Object.entries(SEMANTIC)) {
      expect(value, `SEMANTIC.${key}`).toMatch(HEX_RE);
    }
  });

  it("all TREND values are valid hex", () => {
    for (const [key, value] of Object.entries(TREND)) {
      expect(value, `TREND.${key}`).toMatch(HEX_RE);
    }
  });

  it("all LAYER values are valid hex", () => {
    for (const [key, value] of Object.entries(LAYER)) {
      expect(value, `LAYER.${key}`).toMatch(HEX_RE);
    }
  });

  it("all PRICE_STOPS values are valid hex", () => {
    for (const [key, value] of Object.entries(PRICE_STOPS)) {
      expect(value, `PRICE_STOPS.${key}`).toMatch(HEX_RE);
    }
  });

  it("all SCORE values are valid hex", () => {
    for (const [key, value] of Object.entries(SCORE)) {
      expect(value, `SCORE.${key}`).toMatch(HEX_RE);
    }
  });

  it("all OVERLAY values are valid rgba strings", () => {
    for (const [key, value] of Object.entries(OVERLAY)) {
      expect(value, `OVERLAY.${key}`).toMatch(RGBA_RE);
    }
  });

  it("PRICE_STOPS.max is the D33 danger red #ef4444", () => {
    expect(PRICE_STOPS.max).toBe("#ef4444");
  });
});
