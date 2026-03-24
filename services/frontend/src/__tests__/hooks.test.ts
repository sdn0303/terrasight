import { renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { createQueryWrapper } from "./test-utils";

// ─── Mocks ───────────────────────────────────────────

const mockFetchHealth = vi.fn();
const mockFetchStats = vi.fn();
const mockFetchScore = vi.fn();
const mockFetchTrend = vi.fn();
const mockFetchAreaData = vi.fn();

vi.mock("@/lib/api", () => ({
  fetchHealth: (...args: unknown[]) => mockFetchHealth(...args),
  fetchStats: (...args: unknown[]) => mockFetchStats(...args),
  fetchScore: (...args: unknown[]) => mockFetchScore(...args),
  fetchTrend: (...args: unknown[]) => mockFetchTrend(...args),
  fetchAreaData: (...args: unknown[]) => mockFetchAreaData(...args),
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

const SCORE_FIXTURE = {
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
});

// ─── useHealth ───────────────────────────────────────

describe("useHealth", () => {
  it("fetches and returns health data", async () => {
    mockFetchHealth.mockResolvedValueOnce(HEALTH_OK);
    const { useHealth } = await import(
      "@/features/health/api/use-health"
    );
    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(() => useHealth(), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toEqual(HEALTH_OK);
  });

  it("forwards AbortSignal to fetchHealth", async () => {
    mockFetchHealth.mockResolvedValueOnce(HEALTH_OK);
    const { useHealth } = await import(
      "@/features/health/api/use-health"
    );
    const { wrapper } = createQueryWrapper();

    renderHook(() => useHealth(), { wrapper });

    await waitFor(() =>
      expect(mockFetchHealth).toHaveBeenCalledWith(expect.any(AbortSignal)),
    );
  });
});

// ─── useStats ────────────────────────────────────────

describe("useStats", () => {
  it("fetches stats when bbox is provided", async () => {
    mockFetchStats.mockResolvedValueOnce(STATS_FIXTURE);
    const { useStats } = await import(
      "@/features/stats/api/use-stats"
    );
    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(() => useStats(BBOX, 12), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data?.land_price.count).toBe(45);
  });

  it("does not fetch when bbox is null (enabled: false)", async () => {
    const { useStats } = await import(
      "@/features/stats/api/use-stats"
    );
    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(() => useStats(null, 12), { wrapper });

    expect(result.current.fetchStatus).toBe("idle");
    expect(mockFetchStats).not.toHaveBeenCalled();
  });

  it("forwards AbortSignal to fetchStats", async () => {
    mockFetchStats.mockResolvedValueOnce(STATS_FIXTURE);
    const { useStats } = await import(
      "@/features/stats/api/use-stats"
    );
    const { wrapper } = createQueryWrapper();

    renderHook(() => useStats(BBOX, 12), { wrapper });

    await waitFor(() =>
      expect(mockFetchStats).toHaveBeenCalledWith(BBOX, expect.any(AbortSignal)),
    );
  });
});

// ─── useScore ────────────────────────────────────────

describe("useScore", () => {
  it("fetches score when lat/lng are provided", async () => {
    mockFetchScore.mockResolvedValueOnce(SCORE_FIXTURE);
    const { useScore } = await import(
      "@/features/score/api/use-score"
    );
    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(() => useScore(35.681, 139.767), {
      wrapper,
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data?.score).toBe(72);
  });

  it("does not fetch when lat is null", async () => {
    const { useScore } = await import(
      "@/features/score/api/use-score"
    );
    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(() => useScore(null, 139.767), {
      wrapper,
    });

    expect(result.current.fetchStatus).toBe("idle");
  });

  it("does not fetch when lng is null", async () => {
    const { useScore } = await import(
      "@/features/score/api/use-score"
    );
    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(() => useScore(35.681, null), {
      wrapper,
    });

    expect(result.current.fetchStatus).toBe("idle");
  });

  it("forwards AbortSignal to fetchScore", async () => {
    mockFetchScore.mockResolvedValueOnce(SCORE_FIXTURE);
    const { useScore } = await import(
      "@/features/score/api/use-score"
    );
    const { wrapper } = createQueryWrapper();

    renderHook(() => useScore(35.681, 139.767), { wrapper });

    await waitFor(() =>
      expect(mockFetchScore).toHaveBeenCalledWith(
        35.681,
        139.767,
        expect.any(AbortSignal),
      ),
    );
  });
});

// ─── useTrend ────────────────────────────────────────

describe("useTrend", () => {
  it("fetches trend when lat/lng are provided", async () => {
    mockFetchTrend.mockResolvedValueOnce(TREND_FIXTURE);
    const { useTrend } = await import(
      "@/features/trend/api/use-trend"
    );
    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(() => useTrend(35.681, 139.767), {
      wrapper,
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data?.direction).toBe("up");
  });

  it("does not fetch when lat is null", async () => {
    const { useTrend } = await import(
      "@/features/trend/api/use-trend"
    );
    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(() => useTrend(null, 139.767), {
      wrapper,
    });

    expect(result.current.fetchStatus).toBe("idle");
  });

  it("passes years parameter through", async () => {
    mockFetchTrend.mockResolvedValueOnce(TREND_FIXTURE);
    const { useTrend } = await import(
      "@/features/trend/api/use-trend"
    );
    const { wrapper } = createQueryWrapper();

    renderHook(() => useTrend(35.681, 139.767, 10), { wrapper });

    await waitFor(() =>
      expect(mockFetchTrend).toHaveBeenCalledWith(
        35.681,
        139.767,
        10,
        expect.any(AbortSignal),
      ),
    );
  });
});

// ─── useAreaData ─────────────────────────────────────

describe("useAreaData", () => {
  it("fetches area data when bbox and layers are provided", async () => {
    const areaFixture = { landprice: { type: "FeatureCollection", features: [] } };
    mockFetchAreaData.mockResolvedValueOnce(areaFixture);
    const { useAreaData } = await import(
      "@/features/area-data/api/use-area-data"
    );
    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(
      () => useAreaData(BBOX, ["landprice"], 12),
      { wrapper },
    );

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toEqual(areaFixture);
  });

  it("does not fetch when bbox is null", async () => {
    const { useAreaData } = await import(
      "@/features/area-data/api/use-area-data"
    );
    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(
      () => useAreaData(null, ["landprice"], 12),
      { wrapper },
    );

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
    const areaFixture = { landprice: { type: "FeatureCollection", features: [] } };
    mockFetchAreaData.mockResolvedValueOnce(areaFixture);
    const { useAreaData } = await import(
      "@/features/area-data/api/use-area-data"
    );
    const { wrapper } = createQueryWrapper();

    renderHook(() => useAreaData(BBOX, ["landprice"], 12), { wrapper });

    await waitFor(() =>
      expect(mockFetchAreaData).toHaveBeenCalledWith(
        BBOX,
        ["landprice"],
        12,
        expect.any(AbortSignal),
      ),
    );
  });
});
