import { describe, expect, it } from "vitest";
import { canonicalLayerId } from "@/lib/layer-ids";

describe("canonicalLayerId", () => {
  it("normalizes underscore IDs to hyphen-case", () => {
    expect(canonicalLayerId("admin_boundary")).toBe("admin-boundary");
    expect(canonicalLayerId("flood_history")).toBe("flood-history");
    expect(canonicalLayerId("steep_slope")).toBe("steep-slope");
    expect(canonicalLayerId("land_price_ts")).toBe("land-price-ts");
    expect(canonicalLayerId("population_mesh")).toBe("population-mesh");
  });

  it("passes through already-canonical IDs unchanged", () => {
    expect(canonicalLayerId("geology")).toBe("geology");
    expect(canonicalLayerId("landform")).toBe("landform");
    expect(canonicalLayerId("admin-boundary")).toBe("admin-boundary");
    expect(canonicalLayerId("did")).toBe("did");
  });

  it("passes through unknown IDs unchanged", () => {
    expect(canonicalLayerId("unknown_layer")).toBe("unknown_layer");
  });
});
