import { beforeEach, describe, expect, it } from "vitest";
import { useFilterStore } from "@/stores/filter-store";

describe("useFilterStore", () => {
  beforeEach(() => {
    useFilterStore.getState().reset();
  });

  it("starts with default state", () => {
    const s = useFilterStore.getState();
    expect(s.area.prefecture).toBe("東京都");
    expect(s.area.cities).toEqual([]);
    expect(s.area.customPolygon).toBeNull();
    expect(s.criteria.tlsMin).toBe(0);
    expect(s.criteria.riskMax).toBe("high");
    expect(s.criteria.priceRange).toEqual([0, 10_000_000]);
    expect(s.zoning.zones).toEqual([]);
    expect(s.zoning.stationMaxDistanceM).toBe(2000);
    expect(s.preset).toBe("balance");
  });

  it("isActive returns false when at defaults", () => {
    expect(useFilterStore.getState().isActive()).toBe(false);
  });

  it("isActive returns true after setting any non-default", () => {
    useFilterStore.getState().setCriteria({ tlsMin: 60 });
    expect(useFilterStore.getState().isActive()).toBe(true);
  });

  it("activeCount counts modified filters", () => {
    const s = useFilterStore.getState();
    expect(s.activeCount()).toBe(0);
    s.setCriteria({ tlsMin: 60 });
    expect(useFilterStore.getState().activeCount()).toBe(1);
    s.setZoning({ zones: ["商業"] });
    expect(useFilterStore.getState().activeCount()).toBe(2);
    s.setPreset("investment");
    expect(useFilterStore.getState().activeCount()).toBe(3);
  });

  it("setArea merges partial updates", () => {
    useFilterStore.getState().setArea({ cities: ["渋谷区"] });
    const s = useFilterStore.getState();
    expect(s.area.prefecture).toBe("東京都"); // unchanged
    expect(s.area.cities).toEqual(["渋谷区"]);
  });

  it("setCriteria merges partial updates", () => {
    const s = useFilterStore.getState();
    s.setCriteria({ tlsMin: 70 });
    expect(useFilterStore.getState().criteria.tlsMin).toBe(70);
    expect(useFilterStore.getState().criteria.riskMax).toBe("high");
  });

  it("reset restores defaults", () => {
    const s = useFilterStore.getState();
    s.setCriteria({ tlsMin: 80 });
    s.setZoning({ zones: ["商業"] });
    s.reset();
    expect(useFilterStore.getState().isActive()).toBe(false);
  });

  it("toQueryParams includes only non-default fields", () => {
    const s = useFilterStore.getState();
    s.setCriteria({ tlsMin: 60, riskMax: "mid" });
    s.setZoning({ zones: ["商業", "近商"] });
    const p = useFilterStore.getState().toQueryParams();
    expect(p.tls_min).toBe("60");
    expect(p.risk_max).toBe("mid");
    expect(p.zones).toBe("商業,近商");
    expect(p.price_min).toBeUndefined();
    expect(p.price_max).toBeUndefined();
    expect(p.station_max).toBeUndefined();
    expect(p.preset).toBeUndefined();
    expect(p.cities).toBeUndefined();
  });

  it("toQueryParams returns empty object at defaults", () => {
    expect(useFilterStore.getState().toQueryParams()).toEqual({});
  });

  it("isActive matches activeCount > 0", () => {
    const s = useFilterStore.getState();
    expect(s.isActive()).toBe(s.activeCount() > 0);
    s.setCriteria({ tlsMin: 50 });
    expect(useFilterStore.getState().isActive()).toBe(
      useFilterStore.getState().activeCount() > 0,
    );
  });
});
