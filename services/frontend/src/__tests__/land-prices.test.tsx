import { renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { queryKeys } from "@/lib/query-keys";
import { LandPriceTimeSeriesResponse } from "@/lib/api/schemas/land-prices";
import { createQueryWrapper } from "./test-utils";

// ─── Mocks ───────────────────────────────────────────

const mockTypedGet = vi.fn();

vi.mock("@/lib/api", () => ({
  typedGet: (...args: unknown[]) => mockTypedGet(...args),
  api: {},
  BBox: {},
}));

// useMediaQuery mock retained for forward compatibility when component
// tests are re-enabled via integration test harness.
const mockUseMediaQuery = vi.fn().mockReturnValue(false);
vi.mock("@/hooks/use-media-query", () => ({
  useMediaQuery: (...args: unknown[]) => mockUseMediaQuery(...args),
}));

// ─── Fixtures ────────────────────────────────────────

const BBOX = { south: 35.6, west: 139.7, north: 35.8, east: 139.9 };

const VALID_LAND_PRICE_FC = {
  type: "FeatureCollection" as const,
  truncated: false,
  count: 1,
  limit: 5000,
  features: [
    {
      type: "Feature" as const,
      geometry: {
        type: "Polygon" as const,
        coordinates: [
          [
            [139.767, 35.681],
            [139.768, 35.681],
            [139.768, 35.682],
            [139.767, 35.682],
            [139.767, 35.681],
          ],
        ],
      },
      properties: {
        id: 1,
        price_per_sqm: 1200000,
        address: "千代田区丸の内1-1-1",
        land_use: "商業",
        year: 2024,
      },
    },
  ],
};

beforeEach(() => {
  vi.clearAllMocks();
  mockTypedGet.mockReset();
  // Default to desktop (non-mobile) for component tests
  mockUseMediaQuery.mockReturnValue(false);
});

// ─── LandPriceTimeSeriesResponse schema ──────────────

describe("LandPriceTimeSeriesResponse schema", () => {
  it("parses a valid LayerResponseDto with Polygon geometry", () => {
    const result = LandPriceTimeSeriesResponse.parse(VALID_LAND_PRICE_FC);
    expect(result.type).toBe("FeatureCollection");
    expect(result.features).toHaveLength(1);
    expect(result.features[0]?.properties.price_per_sqm).toBe(1200000);
    expect(result.truncated).toBe(false);
    expect(result.count).toBe(1);
    expect(result.limit).toBe(5000);
  });

  it("parses a FeatureCollection with null land_use", () => {
    const data = {
      ...VALID_LAND_PRICE_FC,
      features: [
        {
          ...VALID_LAND_PRICE_FC.features[0],
          properties: {
            ...VALID_LAND_PRICE_FC.features[0]?.properties,
            land_use: null,
          },
        },
      ],
    };
    const result = LandPriceTimeSeriesResponse.parse(data);
    expect(result.features[0]?.properties.land_use).toBeNull();
  });

  it("parses an empty FeatureCollection", () => {
    const data = {
      type: "FeatureCollection" as const,
      features: [],
      truncated: false,
      count: 0,
      limit: 5000,
    };
    const result = LandPriceTimeSeriesResponse.parse(data);
    expect(result.features).toHaveLength(0);
  });

  it("rejects when type is not FeatureCollection", () => {
    const data = { type: "Feature", geometry: null, properties: {} };
    expect(() => LandPriceTimeSeriesResponse.parse(data)).toThrow();
  });

  it("rejects a feature missing required property price_per_sqm", () => {
    const data = {
      type: "FeatureCollection" as const,
      truncated: false,
      count: 1,
      limit: 5000,
      features: [
        {
          type: "Feature" as const,
          geometry: {
            type: "Polygon" as const,
            coordinates: [
              [
                [139.767, 35.681],
                [139.768, 35.681],
                [139.768, 35.682],
                [139.767, 35.682],
                [139.767, 35.681],
              ],
            ],
          },
          properties: {
            id: 1,
            address: "千代田区",
            land_use: null,
            year: 2024,
            // price_per_sqm omitted
          },
        },
      ],
    };
    expect(() => LandPriceTimeSeriesResponse.parse(data)).toThrow();
  });

  it("rejects when truncated field is missing", () => {
    const { truncated: _truncated, ...withoutTruncated } = VALID_LAND_PRICE_FC;
    expect(() => LandPriceTimeSeriesResponse.parse(withoutTruncated)).toThrow();
  });
});

// ─── queryKeys.landPrices ─────────────────────────────

describe("queryKeys.landPrices", () => {
  it("all is a static array with the correct base key", () => {
    expect(queryKeys.landPrices.all).toEqual(["land-prices"]);
  });

  it("byYear includes bbox and year in the key", () => {
    const key = queryKeys.landPrices.byYear(BBOX, 2024);
    expect(key).toEqual([
      "land-prices",
      BBOX.south,
      BBOX.west,
      BBOX.north,
      BBOX.east,
      2024,
    ]);
  });

  it("byYear differentiates by year", () => {
    const key2024 = queryKeys.landPrices.byYear(BBOX, 2024);
    const key2023 = queryKeys.landPrices.byYear(BBOX, 2023);
    expect(key2024).not.toEqual(key2023);
  });

  it("byYear differentiates by bbox", () => {
    const bbox2 = { south: 35.0, west: 139.0, north: 35.2, east: 139.2 };
    const key1 = queryKeys.landPrices.byYear(BBOX, 2024);
    const key2 = queryKeys.landPrices.byYear(bbox2, 2024);
    expect(key1).not.toEqual(key2);
  });
});

// ─── useLandPrices hook ───────────────────────────────

describe("useLandPrices", () => {
  it("fetches land prices when bbox and year are provided", async () => {
    mockTypedGet.mockResolvedValueOnce(VALID_LAND_PRICE_FC);
    const { useLandPrices } = await import(
      "@/features/land-prices/api/use-land-prices"
    );
    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(() => useLandPrices(BBOX, 2024, 12), {
      wrapper,
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data?.type).toBe("FeatureCollection");
    expect(result.current.data?.features).toHaveLength(1);
  });

  it("does not fetch when bbox is null (enabled: false)", async () => {
    const { useLandPrices } = await import(
      "@/features/land-prices/api/use-land-prices"
    );
    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(() => useLandPrices(null, 2024, 12), {
      wrapper,
    });

    expect(result.current.fetchStatus).toBe("idle");
    expect(mockTypedGet).not.toHaveBeenCalled();
  });

  it("forwards AbortSignal and zoom to fetchLandPrices", async () => {
    mockTypedGet.mockResolvedValueOnce(VALID_LAND_PRICE_FC);
    const { useLandPrices } = await import(
      "@/features/land-prices/api/use-land-prices"
    );
    const { wrapper } = createQueryWrapper();

    renderHook(() => useLandPrices(BBOX, 2024, 12), { wrapper });

    await waitFor(() =>
      expect(mockTypedGet).toHaveBeenCalledWith(
        expect.anything(),
        "api/v1/land-prices",
        expect.objectContaining({
          south: String(BBOX.south),
          west: String(BBOX.west),
          north: String(BBOX.north),
          east: String(BBOX.east),
          year: "2024",
          zoom: "12",
        }),
        expect.any(AbortSignal),
      ),
    );
  });

  it("re-fetches when year changes", async () => {
    mockTypedGet.mockResolvedValue(VALID_LAND_PRICE_FC);
    const { useLandPrices } = await import(
      "@/features/land-prices/api/use-land-prices"
    );
    const { wrapper } = createQueryWrapper();

    let year = 2024;
    const { rerender } = renderHook(() => useLandPrices(BBOX, year, 12), {
      wrapper,
    });

    await waitFor(() => expect(mockTypedGet).toHaveBeenCalledTimes(1));

    year = 2023;
    rerender();

    await waitFor(() => expect(mockTypedGet).toHaveBeenCalledTimes(2));
    expect(mockTypedGet).toHaveBeenLastCalledWith(
      expect.anything(),
      "api/v1/land-prices",
      expect.objectContaining({ year: "2023" }),
      expect.any(AbortSignal),
    );
  });

  it("returns isError true when the API call rejects", async () => {
    // The hook has retry: 1, so reject both the initial attempt and the single retry
    mockTypedGet
      .mockRejectedValueOnce(new Error("Network error"))
      .mockRejectedValueOnce(new Error("Network error"));
    const { useLandPrices } = await import(
      "@/features/land-prices/api/use-land-prices"
    );
    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(() => useLandPrices(BBOX, 2024, 12), {
      wrapper,
    });

    await waitFor(() => expect(result.current.isError).toBe(true), {
      timeout: 5000,
    });
  });

  it("disables the query when zoom < 10 and does not call fetchLandPrices", async () => {
    const { useLandPrices } = await import(
      "@/features/land-prices/api/use-land-prices"
    );
    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(() => useLandPrices(BBOX, 2024, 8), {
      wrapper,
    });

    // enabled: zoom >= 10 is false, so query stays idle
    expect(result.current.fetchStatus).toBe("idle");
    expect(mockTypedGet).not.toHaveBeenCalled();
  });
});

// LandPriceExtrusionLayer tests require react-map-gl/mapbox subpath
// which is not resolvable in the vitest jsdom environment.
// Component is verified via integration test instead.

