import { describe, it, expect } from "vitest";
import { isBBoxValid } from "@/lib/api/bbox-guard";

describe("isBBoxValid", () => {
  it("accepts a small bbox well within 0.5 degrees on both sides", () => {
    expect(
      isBBoxValid({ south: 35.6, west: 139.6, north: 35.8, east: 139.8 }),
    ).toBe(true);
  });

  it("rejects bbox exceeding max side on north-south axis", () => {
    expect(
      isBBoxValid({ south: 35.0, west: 139.6, north: 35.6, east: 139.8 }),
    ).toBe(false);
  });

  it("rejects bbox exceeding max side on east-west axis", () => {
    expect(
      isBBoxValid({ south: 35.6, west: 139.0, north: 35.8, east: 139.6 }),
    ).toBe(false);
  });

  it("accepts bbox at the exact boundary of 0.5 on both axes", () => {
    expect(
      isBBoxValid({ south: 35.5, west: 139.5, north: 36.0, east: 140.0 }),
    ).toBe(true);
  });

  it("rejects when only east-west side exceeds the max", () => {
    expect(
      isBBoxValid({ south: 35.6, west: 139.0, north: 35.9, east: 139.6 }),
    ).toBe(false);
  });

  it("rejects when only north-south side exceeds the max", () => {
    expect(
      isBBoxValid({ south: 35.0, west: 139.6, north: 35.51, east: 139.9 }),
    ).toBe(false);
  });
});
