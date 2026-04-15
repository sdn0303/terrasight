import { describe, expect, it } from "vitest";
import { getPrefectureCodes } from "@/lib/prefecture";

describe("getPrefectureCodes", () => {
  it("returns '13' for a bbox centered on Shinjuku", () => {
    // Shinjuku is roughly 35.69°N, 139.70°E — well within Tokyo (code 13)
    const codes = getPrefectureCodes({
      south: 35.68,
      west: 139.69,
      north: 35.7,
      east: 139.71,
    });
    expect(codes).toContain("13");
  });

  it("returns both Tokyo (13) and Kanagawa (14) for a bbox spanning their border", () => {
    // Tokyo southern boundary ~35.50, Kanagawa northern boundary ~35.67
    // A bbox straddling ~35.55 will intersect both
    const codes = getPrefectureCodes({
      south: 35.48,
      west: 139.4,
      north: 35.68,
      east: 139.6,
    });
    expect(codes).toContain("13");
    expect(codes).toContain("14");
  });

  it("returns empty array for a bbox in deep ocean with no prefecture overlap", () => {
    // Far Pacific ocean — no prefecture bbox reaches here
    const codes = getPrefectureCodes({
      south: 10.0,
      west: 160.0,
      north: 15.0,
      east: 165.0,
    });
    expect(codes).toHaveLength(0);
  });

  it("returns only Okinawa (47) for a bbox over Naha", () => {
    // Naha is ~26.2°N, 127.7°E — within Okinawa but not any other prefecture
    const codes = getPrefectureCodes({
      south: 26.1,
      west: 127.6,
      north: 26.3,
      east: 127.8,
    });
    expect(codes).toContain("47");
    expect(codes).not.toContain("46"); // Kagoshima does not extend this far west/south
  });
});
