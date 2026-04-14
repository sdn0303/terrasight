/**
 * Canonical layer ID normalization.
 *
 * UI code (layers.ts, themes.ts, stores) uses underscore IDs for
 * backwards compatibility. WASM manifest, FGB filenames, and worker
 * messages use hyphen-case.
 *
 * Scope: This normalizes at WASM/FGB boundaries only. UI-side code
 * continues using underscore IDs. A full system-wide rename is out
 * of scope for this module.
 */

const ID_NORMALIZE: Record<string, string> = {
  admin_boundary: "admin-boundary",
  flood_history: "flood-history",
  steep_slope: "steep-slope",
  land_price_ts: "land-price-ts",
  population_mesh: "population-mesh",
};

/** Convert any layer ID variant to canonical hyphen-case form. */
export function canonicalLayerId(id: string): string {
  return ID_NORMALIZE[id] ?? id;
}
