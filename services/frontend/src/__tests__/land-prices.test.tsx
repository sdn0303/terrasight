import { render, renderHook, screen, waitFor } from "@testing-library/react";
import React from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { queryKeys } from "@/lib/query-keys";
import { LandPriceTimeSeriesResponse } from "@/lib/api/schemas/land-prices";
import { createQueryWrapper } from "./test-utils";

// ─── Mocks ───────────────────────────────────────────

const mockFetchLandPrices = vi.fn();

vi.mock("@/lib/api", () => ({
  fetchLandPrices: (...args: unknown[]) => mockFetchLandPrices(...args),
}));

const mockUseMediaQuery = vi.fn().mockReturnValue(false);
vi.mock("@/hooks/use-media-query", () => ({
  useMediaQuery: (...args: unknown[]) => mockUseMediaQuery(...args),
}));

vi.mock("react-map-gl", () => ({
  Source: ({
    children,
    ...props
  }: {
    children?: React.ReactNode;
    [key: string]: unknown;
  }) =>
    React.createElement(
      "div",
      { "data-testid": "maplibre-source", ...props },
      children,
    ),
  Layer: (props: { id?: string; [key: string]: unknown }) =>
    React.createElement("div", {
      "data-testid": "maplibre-layer",
      "data-id": props.id,
    }),
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
    expect(key).toEqual(["land-prices", BBOX, 2024]);
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
    mockFetchLandPrices.mockResolvedValueOnce(VALID_LAND_PRICE_FC);
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
    expect(mockFetchLandPrices).not.toHaveBeenCalled();
  });

  it("forwards AbortSignal and zoom to fetchLandPrices", async () => {
    mockFetchLandPrices.mockResolvedValueOnce(VALID_LAND_PRICE_FC);
    const { useLandPrices } = await import(
      "@/features/land-prices/api/use-land-prices"
    );
    const { wrapper } = createQueryWrapper();

    renderHook(() => useLandPrices(BBOX, 2024, 12), { wrapper });

    await waitFor(() =>
      expect(mockFetchLandPrices).toHaveBeenCalledWith(
        BBOX,
        2024,
        12,
        expect.any(AbortSignal),
      ),
    );
  });

  it("re-fetches when year changes", async () => {
    mockFetchLandPrices.mockResolvedValue(VALID_LAND_PRICE_FC);
    const { useLandPrices } = await import(
      "@/features/land-prices/api/use-land-prices"
    );
    const { wrapper } = createQueryWrapper();

    let year = 2024;
    const { rerender } = renderHook(() => useLandPrices(BBOX, year, 12), {
      wrapper,
    });

    await waitFor(() => expect(mockFetchLandPrices).toHaveBeenCalledTimes(1));

    year = 2023;
    rerender();

    await waitFor(() => expect(mockFetchLandPrices).toHaveBeenCalledTimes(2));
    expect(mockFetchLandPrices).toHaveBeenLastCalledWith(
      BBOX,
      2023,
      12,
      expect.any(AbortSignal),
    );
  });

  it("returns isError true when the API call rejects", async () => {
    // The hook has retry: 1, so reject both the initial attempt and the single retry
    mockFetchLandPrices
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
    expect(mockFetchLandPrices).not.toHaveBeenCalled();
  });
});

// ─── LandPriceYearSlider ──────────────────────────────

describe("LandPriceYearSlider", () => {
  it("returns null when visible is false", async () => {
    const { LandPriceYearSlider } = await import(
      "@/components/map/land-price-year-slider"
    );
    const { container } = render(
      <LandPriceYearSlider visible={false} value={2024} onChange={vi.fn()} />,
    );
    expect(container.innerHTML).toBe("");
  });

  it("renders desktop slider with aria-label and range input when visible", async () => {
    // desktop: mockUseMediaQuery returns false (not mobile)
    mockUseMediaQuery.mockReturnValue(false);
    const { LandPriceYearSlider } = await import(
      "@/components/map/land-price-year-slider"
    );
    render(
      <LandPriceYearSlider visible={true} value={2024} onChange={vi.fn()} />,
    );

    expect(
      screen.getByRole("group", { name: "地価公示年度選択" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("slider")).toBeInTheDocument();
  });

  it("renders mobile button bar with 5 year buttons and marks active year as pressed", async () => {
    // mobile: mockUseMediaQuery returns true
    mockUseMediaQuery.mockReturnValue(true);
    const { LandPriceYearSlider } = await import(
      "@/components/map/land-price-year-slider"
    );
    render(
      <LandPriceYearSlider visible={true} value={2024} onChange={vi.fn()} />,
    );

    const buttons = screen.getAllByRole("button");
    expect(buttons).toHaveLength(5); // 2020 through 2024

    const activeButton = screen.getByRole("button", { name: "2024年" });
    expect(activeButton).toHaveAttribute("aria-pressed", "true");

    const inactiveButton = screen.getByRole("button", { name: "2020年" });
    expect(inactiveButton).toHaveAttribute("aria-pressed", "false");
  });

  it("shows error state with role=alert and error message when isError is true", async () => {
    mockUseMediaQuery.mockReturnValue(false);
    const { LandPriceYearSlider } = await import(
      "@/components/map/land-price-year-slider"
    );
    render(
      <LandPriceYearSlider
        visible={true}
        isError={true}
        value={2024}
        onChange={vi.fn()}
      />,
    );

    expect(screen.getByRole("alert")).toBeInTheDocument();
    expect(screen.getByText("データ取得エラー")).toBeInTheDocument();
  });

  it("shows empty state message when featureCount is 0 and not fetching", async () => {
    mockUseMediaQuery.mockReturnValue(false);
    const { LandPriceYearSlider } = await import(
      "@/components/map/land-price-year-slider"
    );
    render(
      <LandPriceYearSlider
        visible={true}
        featureCount={0}
        isFetching={false}
        value={2024}
        onChange={vi.fn()}
      />,
    );

    expect(screen.getByText("このエリアにデータなし")).toBeInTheDocument();
  });

  it("shows zoom prompt when isZoomTooLow is true", async () => {
    mockUseMediaQuery.mockReturnValue(false);
    const { LandPriceYearSlider } = await import(
      "@/components/map/land-price-year-slider"
    );
    render(
      <LandPriceYearSlider
        visible={true}
        isZoomTooLow={true}
        value={2024}
        onChange={vi.fn()}
      />,
    );

    expect(screen.getByText("ズームインしてください")).toBeInTheDocument();
  });

  it("renders pulsing indicator span with aria-hidden when isFetching is true", async () => {
    mockUseMediaQuery.mockReturnValue(false);
    const { LandPriceYearSlider } = await import(
      "@/components/map/land-price-year-slider"
    );
    render(
      <LandPriceYearSlider
        visible={true}
        isFetching={true}
        value={2024}
        onChange={vi.fn()}
      />,
    );

    // The pulsing dot is rendered as an aria-hidden span with inline border-radius: 50%
    const hiddenSpans = Array.from(
      document.querySelectorAll<HTMLSpanElement>('span[aria-hidden="true"]'),
    );
    const pulsingDot = hiddenSpans.find(
      (el) => el.style.borderRadius === "50%",
    );
    expect(pulsingDot).toBeDefined();
  });
});

// ─── LandPriceExtrusionLayer ──────────────────────────

describe("LandPriceExtrusionLayer", () => {
  it("returns null when visible is false", async () => {
    mockUseMediaQuery.mockReturnValue(false);
    const { LandPriceExtrusionLayer } = await import(
      "@/components/map/layers/land-price-extrusion-layer"
    );
    const { container } = render(
      <LandPriceExtrusionLayer
        visible={false}
        data={VALID_LAND_PRICE_FC}
        selectedYear={2024}
      />,
    );
    expect(container.innerHTML).toBe("");
  });

  it("returns null when data has 0 features", async () => {
    mockUseMediaQuery.mockReturnValue(false);
    const { LandPriceExtrusionLayer } = await import(
      "@/components/map/layers/land-price-extrusion-layer"
    );
    const emptyFC = { type: "FeatureCollection" as const, features: [] };
    const { container } = render(
      <LandPriceExtrusionLayer
        visible={true}
        data={emptyFC}
        selectedYear={2024}
      />,
    );
    expect(container.innerHTML).toBe("");
  });

  it("renders Source when visible with valid polygon data on desktop", async () => {
    // desktop: mockUseMediaQuery returns false (not mobile)
    mockUseMediaQuery.mockReturnValue(false);
    const { LandPriceExtrusionLayer } = await import(
      "@/components/map/layers/land-price-extrusion-layer"
    );
    render(
      <LandPriceExtrusionLayer
        visible={true}
        data={VALID_LAND_PRICE_FC}
        selectedYear={2024}
      />,
    );

    expect(screen.getByTestId("maplibre-source")).toBeInTheDocument();
  });
});
