import { describe, expect, it } from "vitest";
import { LAYERS } from "@/lib/layers";
import {
  FloodProperties,
  LandPriceProperties,
  MedicalProperties,
  SchoolProperties,
  SteepSlopeProperties,
  ZoningProperties,
} from "@/lib/schemas";

const SCHEMA_MAP: Record<string, Record<string, unknown>> = {
  land_price_ts: LandPriceProperties.shape,
  landprice: LandPriceProperties.shape,
  flood: FloodProperties.shape,
  steep_slope: SteepSlopeProperties.shape,
  schools: SchoolProperties.shape,
  medical: MedicalProperties.shape,
  zoning: ZoningProperties.shape,
};

describe("popupField key validation", () => {
  it("all API layer popupField keys exist in their Zod schema", () => {
    const mismatches: string[] = [];

    for (const layer of LAYERS) {
      if (layer.source !== "api" && layer.source !== "timeseries") continue;
      if (!layer.popupFields) continue;

      const schema = SCHEMA_MAP[layer.id];
      if (!schema) continue;

      for (const field of layer.popupFields) {
        if (!(field.key in schema)) {
          mismatches.push(
            `${layer.id}: popupField "${field.key}" not in schema (available: ${Object.keys(schema).join(", ")})`,
          );
        }
      }
    }

    expect(mismatches).toEqual([]);
  });

  it("no popupField has a key called 'id' (internal field)", () => {
    for (const layer of LAYERS) {
      if (!layer.popupFields) continue;
      const hasId = layer.popupFields.some((f) => f.key === "id");
      expect(hasId, `${layer.id} has popupField "id"`).toBe(false);
    }
  });
});
