# X-05: Fix popupField Keys to Match Schema Properties

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix 11 mismatched popupField keys in layers.ts so click-inspect popups show actual data instead of empty fields. Add a validation test to prevent future drift.

**Architecture:** Align `popupFields[].key` in layers.ts with the actual property names from schemas.ts (for API layers) and GeoJSON files (for static layers). Add a test that validates all API layer popupField keys exist in their corresponding Zod schema.

**Tech Stack:** React 19, Zod, Vitest

---

## File Structure

| File | Change | Responsibility |
|------|--------|----------------|
| `services/frontend/src/lib/layers.ts` | Modify | Fix 11 popupField key mismatches |
| `services/frontend/src/__tests__/popup-field-validation.test.ts` | Create | Validate popupField keys match schemas |

---

### Task 1: Fix popupField keys and add validation test

**Files:**
- Modify: `services/frontend/src/lib/layers.ts`
- Create: `services/frontend/src/__tests__/popup-field-validation.test.ts`

- [ ] **Step 1: Write the validation test**

Create `services/frontend/src/__tests__/popup-field-validation.test.ts`:

```typescript
import { describe, expect, it } from "vitest";
import { LAYERS } from "@/lib/layers";
import {
  LandPriceProperties,
  ZoningProperties,
  FloodProperties,
  SteepSlopeProperties,
  SchoolProperties,
  MedicalProperties,
} from "@/lib/schemas";

/** Map of API layer IDs to their Zod schemas */
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
      if (!schema) continue; // Static layers without schemas are OK

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
```

- [ ] **Step 2: Run test to verify it FAILS**

```bash
cd services/frontend && pnpm vitest run src/__tests__/popup-field-validation.test.ts
```

Expected: FAIL — mismatches for landprice (price, change_rate), flood (name, depth), steep_slope (name, type), schools (type), medical (type), zoning (name, far, bcr)

- [ ] **Step 3: Fix all popupField keys in layers.ts**

In `services/frontend/src/lib/layers.ts`, make these exact replacements:

**landprice** (id: "landprice", ~line 60-65):
```
Before: { key: "price", label: "価格", suffix: "円/㎡" },
After:  { key: "price_per_sqm", label: "価格", suffix: "円/㎡" },

Before: { key: "change_rate", label: "変動率", suffix: "%" },
Remove this line entirely (field doesn't exist in schema)
```

**flood** (id: "flood", ~line 123-126):
```
Before: { key: "name", label: "河川名" },
After:  { key: "river_name", label: "河川名" },

Before: { key: "depth", label: "想定浸水深", suffix: "m" },
After:  { key: "depth_rank", label: "浸水深ランク" },
```
Note: depth_rank is 0-5 integer (not meters), so remove the "m" suffix and update label.

**steep_slope** (id: "steep_slope", ~line 138-141):
```
Before: { key: "name", label: "区域名" },
After:  { key: "area_name", label: "区域名" },

Before: { key: "type", label: "種別" },
Remove this line entirely (field doesn't exist in schema)
```

**schools** (id: "schools", ~line 280-283):
```
Before: { key: "type", label: "種別" },
After:  { key: "school_type", label: "種別" },
```

**medical** (id: "medical", ~line 295-298):
```
Before: { key: "type", label: "種別" },
After:  { key: "facility_type", label: "種別" },
```

Also add bed_count since it's in the schema:
```
After facility_type line, add: { key: "bed_count", label: "病床数" },
```

**zoning** (id: "zoning", ~line 368-372):
```
Before: { key: "name", label: "用途地域名" },
After:  { key: "zone_type", label: "用途地域名" },

Before: { key: "far", label: "容積率", suffix: "%" },
After:  { key: "floor_area_ratio", label: "容積率", suffix: "%" },

Before: { key: "bcr", label: "建蔽率", suffix: "%" },
After:  { key: "building_coverage", label: "建蔽率", suffix: "%" },
```

- [ ] **Step 4: Run validation test to verify it PASSES**

```bash
cd services/frontend && pnpm vitest run src/__tests__/popup-field-validation.test.ts
```

Expected: PASS — all popupField keys now match schemas

- [ ] **Step 5: Run full test suite**

```bash
cd services/frontend && pnpm vitest run
```

Expected: All tests PASS

- [ ] **Step 6: Run type check + lint**

```bash
cd services/frontend && pnpm tsc --noEmit && pnpm biome check .
```

Expected: Clean

- [ ] **Step 7: Commit**

```bash
git add services/frontend/src/lib/layers.ts \
       services/frontend/src/__tests__/popup-field-validation.test.ts
git commit -m "fix(layers): align 11 popupField keys with schema properties

Popup click-inspect showed empty fields because popupField keys didn't
match actual API response property names. Fixed: price→price_per_sqm,
name→river_name, depth→depth_rank, name→area_name, type→school_type,
type→facility_type, name→zone_type, far→floor_area_ratio, bcr→building_coverage.

Added validation test to prevent future drift.

Closes X-05 (partial — field alignment done, registry consolidation deferred)."
```

---

## Self-Review Checklist

1. **Spec coverage**: Fix popup empty fields ✅. Add drift prevention test ✅. The full LayerRegistry consolidation (type-safe dispatch maps, schema index) is deferred — this fixes the user-facing bug first.

2. **Placeholder scan**: No TBD/TODO. All replacements specified exactly.

3. **Type consistency**: All replacement keys match Zod schema shape keys exactly. `LandPriceProperties.shape` has `price_per_sqm`, `address`, `land_use`, `year`. `FloodProperties.shape` has `depth_rank`, `river_name`. Etc.
