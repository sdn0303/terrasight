import { describe, expect, it } from "vitest";
import {
  CARD_RADIUS,
  CARD_SHADOW,
  DRAWER_WIDTH,
  GAP,
  LEFT_PANEL_WIDTH,
  MAIN_LEFT_BASE,
  MAIN_LEFT_WITH_PANEL,
  MAIN_RIGHT_BASE,
  MAIN_RIGHT_WITH_DRAWER,
  mapOverlayPaddingPx,
  PAGE_INSET,
  RAIL_WIDTH,
} from "@/lib/layout";

describe("layout constants", () => {
  it("defines page inset as 20", () => {
    expect(PAGE_INSET).toBe(20);
  });

  it("defines rail width as 72", () => {
    expect(RAIL_WIDTH).toBe(72);
  });

  it("defines gap as 16", () => {
    expect(GAP).toBe(16);
  });

  it("defines left panel width as 360", () => {
    expect(LEFT_PANEL_WIDTH).toBe(360);
  });

  it("defines drawer width as 340", () => {
    expect(DRAWER_WIDTH).toBe(340);
  });

  it("computes MAIN_LEFT_BASE as 108", () => {
    expect(MAIN_LEFT_BASE).toBe(PAGE_INSET + RAIL_WIDTH + GAP);
    expect(MAIN_LEFT_BASE).toBe(108);
  });

  it("computes MAIN_LEFT_WITH_PANEL as 484", () => {
    expect(MAIN_LEFT_WITH_PANEL).toBe(MAIN_LEFT_BASE + LEFT_PANEL_WIDTH + GAP);
    expect(MAIN_LEFT_WITH_PANEL).toBe(484);
  });

  it("computes MAIN_RIGHT_BASE as 20", () => {
    expect(MAIN_RIGHT_BASE).toBe(PAGE_INSET);
  });

  it("computes MAIN_RIGHT_WITH_DRAWER as 376", () => {
    expect(MAIN_RIGHT_WITH_DRAWER).toBe(PAGE_INSET + DRAWER_WIDTH + GAP);
    expect(MAIN_RIGHT_WITH_DRAWER).toBe(376);
  });

  it("exposes card radius presets", () => {
    expect(CARD_RADIUS.rail).toBe(22);
    expect(CARD_RADIUS.mainPanel).toBe(24);
    expect(CARD_RADIUS.drawer).toBe(24);
    expect(CARD_RADIUS.bottomSheet).toBe(24);
    expect(CARD_RADIUS.smallChip).toBe(14);
    expect(CARD_RADIUS.button).toBe(12);
    expect(CARD_RADIUS.pill).toBe(999);
  });

  it("exposes card shadow presets", () => {
    expect(CARD_SHADOW.strong).toContain("rgba(0,0,0,0.45)");
    expect(CARD_SHADOW.medium).toContain("rgba(0,0,0,0.35)");
    expect(CARD_SHADOW.subtle).toContain("rgba(0,0,0,0.18)");
  });
});

describe("mapOverlayPaddingPx", () => {
  const viewport = { w: 1920, h: 1080 };

  it("returns base padding when no overlays are open", () => {
    const p = mapOverlayPaddingPx(
      {
        leftPanel: null,
        insight: null,
        bottomSheet: null,
        bottomSheetHeightPct: 40,
      },
      viewport,
    );
    expect(p.left).toBe(MAIN_LEFT_BASE);
    expect(p.right).toBe(MAIN_RIGHT_BASE);
    expect(p.top).toBe(PAGE_INSET);
    expect(p.bottom).toBe(PAGE_INSET);
  });

  it("extends left padding when a left panel is open", () => {
    const p = mapOverlayPaddingPx(
      {
        leftPanel: "finder",
        insight: null,
        bottomSheet: null,
        bottomSheetHeightPct: 40,
      },
      viewport,
    );
    expect(p.left).toBe(MAIN_LEFT_WITH_PANEL);
    expect(p.right).toBe(MAIN_RIGHT_BASE);
  });

  it("extends right padding when the insight drawer is open", () => {
    const p = mapOverlayPaddingPx(
      {
        leftPanel: null,
        insight: { kind: "point", lat: 35, lng: 139 },
        bottomSheet: null,
        bottomSheetHeightPct: 40,
      },
      viewport,
    );
    expect(p.right).toBe(MAIN_RIGHT_WITH_DRAWER);
  });

  it("extends bottom padding when the bottom sheet is open", () => {
    const p = mapOverlayPaddingPx(
      {
        leftPanel: null,
        insight: null,
        bottomSheet: "opportunities",
        bottomSheetHeightPct: 40,
      },
      viewport,
    );
    expect(p.bottom).toBe((40 / 100) * viewport.h + PAGE_INSET);
  });

  it("combines all padding offsets when every overlay is open", () => {
    const p = mapOverlayPaddingPx(
      {
        leftPanel: "finder",
        insight: { kind: "point", lat: 35, lng: 139 },
        bottomSheet: "opportunities",
        bottomSheetHeightPct: 40,
      },
      viewport,
    );
    expect(p.left).toBe(MAIN_LEFT_WITH_PANEL);
    expect(p.right).toBe(MAIN_RIGHT_WITH_DRAWER);
    expect(p.top).toBe(PAGE_INSET);
    expect(p.bottom).toBe((40 / 100) * viewport.h + PAGE_INSET);
  });

  it("returns values directly usable as MapLibre fitBounds padding", () => {
    // Sanity check: padding must NEVER exceed viewport dimensions; this was
    // the regression raised in PR review — the previous shape returned
    // `right = viewport.w - offset`, which for 1920px viewports yielded
    // ~1900 and would over-pad fitBounds.
    const p = mapOverlayPaddingPx(
      {
        leftPanel: "finder",
        insight: { kind: "point", lat: 35, lng: 139 },
        bottomSheet: "opportunities",
        bottomSheetHeightPct: 80,
      },
      viewport,
    );
    expect(p.top).toBeLessThan(viewport.h);
    expect(p.bottom).toBeLessThan(viewport.h);
    expect(p.left).toBeLessThan(viewport.w);
    expect(p.right).toBeLessThan(viewport.w);
  });
});
