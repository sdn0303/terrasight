import { renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { createQueryWrapper } from "./test-utils";

// ─── Mocks ───────────────────────────────────────────

const mockTypedGet = vi.fn();

vi.mock("@/lib/api", () => ({
  typedGet: (...args: unknown[]) => mockTypedGet(...args),
  api: {},
  BBox: {},
}));

// useSpatialEngineReady returns false so useStats exercises the API fallback path
vi.mock("@/hooks/use-spatial-engine", () => ({
  useSpatialEngineReady: () => false,
}));

// ─── Fixtures ────────────────────────────────────────

const HEALTH_OK = {
  status: "ok" as const,
  db_connected: true,
  reinfolib_key_set: false,
  version: "0.1.0",
};

const STATS_FIXTURE = {
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
  zoning_distribution: { 商業地域: 0.35 },
};

const makeAxis = (score: number) => ({
  score,
  weight: 0.2,
  confidence: 0.9,
  sub: [{ id: "sub1", score: score * 0.5, available: true, detail: {} }],
});

const SCORE_FIXTURE = {
  location: { lat: 35.681, lng: 139.767 },
  tls: { score: 72, grade: "A" as const, label: "優良" },
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

const TREND_FIXTURE = {
  location: { address: "千代田区丸の内1", distance_m: 120 },
  data: [
    { year: 2020, price_per_sqm: 1020000 },
    { year: 2024, price_per_sqm: 1200000 },
  ],
  cagr: 0.032,
  direction: "up" as const,
};

const BBOX = { south: 35.6, west: 139.7, north: 35.8, east: 139.9 };

beforeEach(() => {
  vi.clearAllMocks();
  mockTypedGet.mockReset();
});

// ─── useHealth ───────────────────────────────────────

describe("useHealth", () => {
  it("fetches and returns health data", async () => {
    mockTypedGet.mockResolvedValueOnce(HEALTH_OK);
    const { useHealth } = await import("@/features/health/api/use-health");
    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(() => useHealth(), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toEqual(HEALTH_OK);
  });

  it("forwards AbortSignal to fetchHealth", async () => {
    mockTypedGet.mockResolvedValueOnce(HEALTH_OK);
    const { useHealth } = await import("@/features/health/api/use-health");
    const { wrapper } = createQueryWrapper();

    renderHook(() => useHealth(), { wrapper });

    await waitFor(() =>
      expect(mockTypedGet).toHaveBeenCalledWith(
        expect.anything(),
        "api/v1/health",
        undefined,
        expect.any(AbortSignal),
      ),
    );
  });
});

// ─── useStats ────────────────────────────────────────

describe("useStats", () => {
  it("fetches stats when bbox is provided", async () => {
    mockTypedGet.mockResolvedValueOnce(STATS_FIXTURE);
    const { useStats } = await import("@/features/stats/api/use-stats");
    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(() => useStats(BBOX, 12), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data?.land_price.count).toBe(45);
  });

  it("does not fetch when bbox is null (enabled: false)", async () => {
    const { useStats } = await import("@/features/stats/api/use-stats");
    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(() => useStats(null, 12), { wrapper });

    expect(result.current.fetchStatus).toBe("idle");
    expect(mockTypedGet).not.toHaveBeenCalled();
  });

  it("forwards AbortSignal to fetchStats", async () => {
    mockTypedGet.mockResolvedValueOnce(STATS_FIXTURE);
    const { useStats } = await import("@/features/stats/api/use-stats");
    const { wrapper } = createQueryWrapper();

    renderHook(() => useStats(BBOX, 12), { wrapper });

    await waitFor(() =>
      expect(mockTypedGet).toHaveBeenCalledWith(
        expect.anything(),
        "api/v1/stats",
        expect.objectContaining({
          south: String(BBOX.south),
          west: String(BBOX.west),
          north: String(BBOX.north),
          east: String(BBOX.east),
        }),
        expect.any(AbortSignal),
      ),
    );
  });
});

// ─── useScore ────────────────────────────────────────

describe("useScore", () => {
  it("fetches score when lat/lng are provided", async () => {
    mockTypedGet.mockResolvedValueOnce(SCORE_FIXTURE);
    const { useScore } = await import("@/features/score/api/use-score");
    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(() => useScore(35.681, 139.767), {
      wrapper,
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data?.tls.score).toBe(72);
  });

  it("does not fetch when lat is null", async () => {
    const { useScore } = await import("@/features/score/api/use-score");
    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(() => useScore(null, 139.767), {
      wrapper,
    });

    expect(result.current.fetchStatus).toBe("idle");
  });

  it("does not fetch when lng is null", async () => {
    const { useScore } = await import("@/features/score/api/use-score");
    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(() => useScore(35.681, null), {
      wrapper,
    });

    expect(result.current.fetchStatus).toBe("idle");
  });

  it("forwards AbortSignal to fetchScore", async () => {
    mockTypedGet.mockResolvedValueOnce(SCORE_FIXTURE);
    const { useScore } = await import("@/features/score/api/use-score");
    const { wrapper } = createQueryWrapper();

    renderHook(() => useScore(35.681, 139.767), { wrapper });

    await waitFor(() =>
      expect(mockTypedGet).toHaveBeenCalledWith(
        expect.anything(),
        "api/v1/score",
        expect.objectContaining({ lat: "35.681", lng: "139.767", preset: "balance" }),
        expect.any(AbortSignal),
      ),
    );
  });
});

// ─── useTrend ────────────────────────────────────────

describe("useTrend", () => {
  it("fetches trend when lat/lng are provided", async () => {
    mockTypedGet.mockResolvedValueOnce(TREND_FIXTURE);
    const { useTrend } = await import("@/features/trend/api/use-trend");
    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(() => useTrend(35.681, 139.767), {
      wrapper,
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data?.direction).toBe("up");
  });

  it("does not fetch when lat is null", async () => {
    const { useTrend } = await import("@/features/trend/api/use-trend");
    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(() => useTrend(null, 139.767), {
      wrapper,
    });

    expect(result.current.fetchStatus).toBe("idle");
  });

  it("passes years parameter through", async () => {
    mockTypedGet.mockResolvedValueOnce(TREND_FIXTURE);
    const { useTrend } = await import("@/features/trend/api/use-trend");
    const { wrapper } = createQueryWrapper();

    renderHook(() => useTrend(35.681, 139.767, 10), { wrapper });

    await waitFor(() =>
      expect(mockTypedGet).toHaveBeenCalledWith(
        expect.anything(),
        "api/v1/trend",
        expect.objectContaining({ lat: "35.681", lng: "139.767", years: "10" }),
        expect.any(AbortSignal),
      ),
    );
  });
});

// ─── useAreaData ─────────────────────────────────────

describe("useAreaData", () => {
  it("fetches area data when bbox and layers are provided", async () => {
    const areaFixture = {
      landprice: { type: "FeatureCollection", features: [] },
    };
    mockTypedGet.mockResolvedValueOnce(areaFixture);
    const { useAreaData } = await import(
      "@/features/area-data/api/use-area-data"
    );
    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(() => useAreaData(BBOX, ["landprice"], 12), {
      wrapper,
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toEqual(areaFixture);
  });

  it("does not fetch when bbox is null", async () => {
    const { useAreaData } = await import(
      "@/features/area-data/api/use-area-data"
    );
    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(() => useAreaData(null, ["landprice"], 12), {
      wrapper,
    });

    expect(result.current.fetchStatus).toBe("idle");
  });

  it("does not fetch when layers array is empty", async () => {
    const { useAreaData } = await import(
      "@/features/area-data/api/use-area-data"
    );
    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(() => useAreaData(BBOX, [], 12), {
      wrapper,
    });

    expect(result.current.fetchStatus).toBe("idle");
  });

  it("forwards AbortSignal to fetchAreaData", async () => {
    const areaFixture = {
      landprice: { type: "FeatureCollection", features: [] },
    };
    mockTypedGet.mockResolvedValueOnce(areaFixture);
    const { useAreaData } = await import(
      "@/features/area-data/api/use-area-data"
    );
    const { wrapper } = createQueryWrapper();

    renderHook(() => useAreaData(BBOX, ["landprice"], 12), { wrapper });

    await waitFor(() =>
      expect(mockTypedGet).toHaveBeenCalledWith(
        expect.anything(),
        "api/v1/area-data",
        expect.objectContaining({
          south: String(BBOX.south),
          west: String(BBOX.west),
          north: String(BBOX.north),
          east: String(BBOX.east),
          layers: "landprice",
        }),
        expect.any(AbortSignal),
      ),
    );
  });
});
