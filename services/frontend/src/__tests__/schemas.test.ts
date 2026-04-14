import { describe, expect, it } from "vitest";
import { HealthResponse } from "@/lib/api/schemas/health";
import { StatsResponse } from "@/lib/api/schemas/stats";
import { TlsResponse } from "@/lib/api/schemas/score";
import { TrendResponse } from "@/lib/api/schemas/trend";

describe("HealthResponse schema", () => {
  it("parses valid health response", () => {
    const data = {
      status: "ok",
      db_connected: true,
      reinfolib_key_set: false,
      version: "0.1.0",
    };
    expect(HealthResponse.parse(data)).toEqual(data);
  });

  it("rejects invalid status", () => {
    const data = {
      status: "broken",
      db_connected: true,
      reinfolib_key_set: false,
      version: "0.1.0",
    };
    expect(() => HealthResponse.parse(data)).toThrow();
  });
});

describe("TlsResponse schema", () => {
  it("parses valid TLS response", () => {
    const makeAxis = (score: number) => ({
      score,
      weight: 0.2,
      confidence: 0.9,
      sub: [{ id: "sub1", score: score * 0.5, available: true, detail: {} }],
    });
    const data = {
      location: { lat: 35.681, lng: 139.767 },
      tls: { score: 72, grade: "A", label: "優良" },
      axes: {
        disaster: makeAxis(80),
        terrain: makeAxis(75),
        livability: makeAxis(70),
        future: makeAxis(65),
        price: makeAxis(60),
      },
      cross_analysis: {
        value_discovery: 0.8,
        demand_signal: 0.7,
        ground_safety: 0.9,
      },
      metadata: {
        calculated_at: "2026-03-25T10:00:00Z",
        weight_preset: "default",
        data_freshness: "2024",
        disclaimer: "本スコアは参考値です。",
      },
    };
    const result = TlsResponse.parse(data);
    expect(result.tls.score).toBe(72);
    expect(result.tls.grade).toBe("A");
    expect(result.axes.disaster.score).toBe(80);
  });

  it("rejects invalid grade enum value", () => {
    const makeAxis = (score: number) => ({
      score,
      weight: 0.2,
      confidence: 1.0,
      sub: [],
    });
    const data = {
      location: { lat: 35.681, lng: 139.767 },
      tls: { score: 72, grade: "Z", label: "不明" },
      axes: {
        disaster: makeAxis(80),
        terrain: makeAxis(75),
        livability: makeAxis(70),
        future: makeAxis(65),
        price: makeAxis(60),
      },
      cross_analysis: {
        value_discovery: 0.8,
        demand_signal: 0.7,
        ground_safety: 0.9,
      },
      metadata: {
        calculated_at: "2026-03-25T10:00:00Z",
        weight_preset: "default",
        data_freshness: "2024",
        disclaimer: "本スコアは参考値です。",
      },
    };
    expect(() => TlsResponse.parse(data)).toThrow();
  });
});

describe("TrendResponse schema", () => {
  it("parses valid trend response", () => {
    const data = {
      location: { address: "千代田区丸の内1", distance_m: 120 },
      data: [
        { year: 2020, price_per_sqm: 1020000 },
        { year: 2024, price_per_sqm: 1200000 },
      ],
      cagr: 0.032,
      direction: "up" as const,
    };
    const result = TrendResponse.parse(data);
    expect(result.direction).toBe("up");
    expect(result.data).toHaveLength(2);
  });
});

describe("StatsResponse schema", () => {
  it("parses valid stats response", () => {
    const data = {
      land_price: {
        avg_per_sqm: 850000,
        median_per_sqm: 720000,
        min_per_sqm: 320000,
        max_per_sqm: 3200000,
        count: 45,
      },
      risk: {
        flood_area_ratio: 0.15,
        steep_slope_area_ratio: 0.02,
        composite_risk: 0.18,
      },
      facilities: { schools: 12, medical: 28 },
      zoning_distribution: { 商業地域: 0.35, 住居地域: 0.65 },
    };
    const result = StatsResponse.parse(data);
    expect(result.land_price.count).toBe(45);
  });
});
