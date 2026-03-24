import { renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { createQueryWrapper } from "./test-utils";

// ─── Mocks ───────────────────────────────────────────────────────────────────

// Mock flatgeobuf deserialize to yield two dummy features
const mockDeserialize = vi.fn();
vi.mock("flatgeobuf/lib/mjs/geojson", () => ({
  deserialize: (...args: unknown[]) => mockDeserialize(...args),
}));

// Mock layerUrl so we control the constructed URL
vi.mock("@/lib/data-url", () => ({
  layerUrl: (prefCode: string, layerId: string) =>
    `/data/fgb/${prefCode}/${layerId}.fgb`,
}));

// ─── Helpers ─────────────────────────────────────────────────────────────────

/** Build a minimal ReadableStream-like body for fetch mock */
function makeBody(): ReadableStream {
  return new ReadableStream();
}

function makeFetchResponse(ok: boolean, status = 200): Response {
  return {
    ok,
    status,
    body: makeBody(),
  } as unknown as Response;
}

// ─── Test suite ──────────────────────────────────────────────────────────────

// Import after mocks are declared so module resolution picks up mocks
const { useStaticLayer } = await import("@/hooks/use-static-layer");

describe("useStaticLayer", () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it("returns undefined data and does not call fetch when disabled", () => {
    const fetchSpy = vi.spyOn(globalThis, "fetch");
    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(
      () => useStaticLayer("13", "city-boundary", false),
      { wrapper },
    );

    expect(result.current.data).toBeUndefined();
    expect(fetchSpy).not.toHaveBeenCalled();
  });

  it("calls fetch with the correct URL when enabled and returns a FeatureCollection", async () => {
    const PREF_CODE = "13";
    const LAYER_ID = "city-boundary";
    const EXPECTED_URL = `/data/fgb/${PREF_CODE}/${LAYER_ID}.fgb`;

    const fetchSpy = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValue(makeFetchResponse(true));

    // mockDeserialize returns an async iterable yielding two dummy features
    const dummyFeatures = [
      { type: "Feature", geometry: null, properties: {} },
      { type: "Feature", geometry: null, properties: { id: 1 } },
    ];
    mockDeserialize.mockReturnValue(
      (async function* () {
        for (const f of dummyFeatures) yield f;
      })(),
    );

    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(
      () => useStaticLayer(PREF_CODE, LAYER_ID, true),
      { wrapper },
    );

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    // Fetch was called with the correct URL
    expect(fetchSpy).toHaveBeenCalledOnce();
    const [calledUrl] = fetchSpy.mock.calls[0] as [string, RequestInit];
    expect(calledUrl).toBe(EXPECTED_URL);

    // Returned data is a valid FeatureCollection
    expect(result.current.data).toEqual({
      type: "FeatureCollection",
      features: dummyFeatures,
    });
  });

  it("transitions to error state when fetch returns a non-ok response", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValue(
      makeFetchResponse(false, 404),
    );

    const { wrapper } = createQueryWrapper();

    const { result } = renderHook(
      () => useStaticLayer("13", "missing-layer", true),
      { wrapper },
    );

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect((result.current.error as Error).message).toMatch("404");
  });
});
