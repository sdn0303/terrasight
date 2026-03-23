import { describe, expect, it } from "vitest";
import {
  HealthResponse,
  ScoreResponse,
  StatsResponse,
  TrendResponse,
} from "@/lib/schemas";

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

describe("ScoreResponse schema", () => {
  it("parses valid score response", () => {
    const data = {
      score: 72,
      components: {
        trend: { value: 18, max: 25, detail: { cagr_5y: 0.032 } },
        risk: { value: 22, max: 25, detail: { flood_overlap: 0.0 } },
        access: { value: 15, max: 25, detail: { schools_1km: 3 } },
        yield_potential: { value: 17, max: 25, detail: {} },
      },
      metadata: {
        calculated_at: "2026-03-20T10:30:00Z",
        data_freshness: "2024",
        disclaimer: "本スコアは参考値です。",
      },
    };
    const result = ScoreResponse.parse(data);
    expect(result.score).toBe(72);
    expect(result.components.trend.value).toBe(18);
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
