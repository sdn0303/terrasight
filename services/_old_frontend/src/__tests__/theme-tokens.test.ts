import { describe, expect, it } from "vitest";
import {
  GLOW_SHADOW,
  GRADIENT,
  scoreGradient,
  signalGradient,
} from "@/lib/theme-tokens";

describe("theme tokens", () => {
  it("defines brand gradient", () => {
    expect(GRADIENT.brand).toContain("#6366f1");
    expect(GRADIENT.brand).toContain("#8b5cf6");
    expect(GRADIENT.brand).toContain("#06b6d4");
    expect(GRADIENT.brand).toMatch(/^linear-gradient\(135deg/);
  });

  it("defines semantic gradients", () => {
    expect(GRADIENT.success).toContain("#10b981");
    expect(GRADIENT.warn).toContain("#f59e0b");
    expect(GRADIENT.danger).toContain("#ef4444");
    expect(GRADIENT.hot).toContain("#f43f5e");
  });

  it("defines table header gradient", () => {
    expect(GRADIENT.tableHeader).toContain("#312e81");
    expect(GRADIENT.tableHeader).toMatch(/^linear-gradient\(90deg/);
  });

  it("defines glow shadow recipes", () => {
    expect(GLOW_SHADOW.primary).toContain("rgba(99,102,241");
    expect(GLOW_SHADOW.success).toContain("rgba(16,185,129");
    expect(GLOW_SHADOW.hot).toContain("rgba(244,63,94");
  });
});

describe("scoreGradient", () => {
  it("returns danger gradient for low scores", () => {
    expect(scoreGradient(10)).toBe(GRADIENT.danger);
    expect(scoreGradient(39)).toBe(GRADIENT.danger);
  });

  it("returns warn gradient for mid-low scores", () => {
    expect(scoreGradient(40)).toBe(GRADIENT.warn);
    expect(scoreGradient(59)).toBe(GRADIENT.warn);
  });

  it("returns brand gradient for mid-high scores", () => {
    expect(scoreGradient(60)).toBe(GRADIENT.brand);
    expect(scoreGradient(79)).toBe(GRADIENT.brand);
  });

  it("returns success gradient for high scores", () => {
    expect(scoreGradient(80)).toBe(GRADIENT.success);
    expect(scoreGradient(100)).toBe(GRADIENT.success);
  });

  it("clamps out-of-range values", () => {
    expect(scoreGradient(-5)).toBe(GRADIENT.danger);
    expect(scoreGradient(150)).toBe(GRADIENT.success);
  });
});

describe("signalGradient", () => {
  it("returns hot gradient for hot signal", () => {
    expect(signalGradient("hot")).toBe(GRADIENT.hot);
  });

  it("returns warm gradient for warm signal", () => {
    expect(signalGradient("warm")).toBe(GRADIENT.warm);
  });

  it("returns neutral gradient for neutral signal", () => {
    expect(signalGradient("neutral")).toBe(GRADIENT.neutral);
  });

  it("returns cold gradient for cold signal", () => {
    expect(signalGradient("cold")).toBe(GRADIENT.cold);
  });
});
