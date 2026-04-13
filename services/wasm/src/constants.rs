//! Centralized constants for the WASM spatial engine.
//!
//! All magic numbers and string literals referenced in more than one
//! location are collected here to satisfy `proj-no-magic-numbers`.

// ── Layer IDs (canonical form: hyphens and underscores removed) ──
//
// The dataset catalog uses hyphen-case ("land-price"), the backend uses
// underscore-case ("steep_slope"), and legacy WASM code used concatenated
// form ("landprice"). `canonical_layer_id` normalises all three to the same
// key, so these constants only need one form.
pub(crate) const LAYER_LANDPRICE: &str = "landprice";
pub(crate) const LAYER_FLOOD_HISTORY: &str = "floodhistory";
pub(crate) const LAYER_FLOOD: &str = "flood";
pub(crate) const LAYER_STEEP_SLOPE: &str = "steepslope";
pub(crate) const LAYER_ZONING: &str = "zoning";
pub(crate) const LAYER_SCHOOLS: &str = "schools";
pub(crate) const LAYER_MEDICAL: &str = "medical";

// ── GeoJSON property keys ──
pub(crate) const PROP_PRICE_PER_SQM: &str = "price_per_sqm";
pub(crate) const PROP_ZONE_TYPE: &str = "zone_type";

// ── Risk weights (match backend STATS_RISK_WEIGHT_*) ──
pub(crate) const RISK_WEIGHT_FLOOD: f64 = 0.6;
pub(crate) const RISK_WEIGHT_STEEP: f64 = 0.4;

/// Estimated bytes per GeoJSON feature string for capacity pre-allocation.
pub(crate) const GEOJSON_FEATURE_BYTES_ESTIMATE: usize = 256;

/// Normalize a layer ID to its canonical form by removing hyphens and underscores.
///
/// The dataset catalog uses hyphen-case ("land-price"), the backend uses
/// underscore-case ("steep_slope"), while internal WASM code uses concatenated
/// form ("landprice"). This function ensures all three conventions resolve to the
/// same internal key.
pub(crate) fn canonical_layer_id(id: &str) -> String {
    id.replace(['-', '_'], "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_layer_id_normalizes() {
        assert_eq!(canonical_layer_id("land-price"), "landprice");
        assert_eq!(canonical_layer_id("steep_slope"), "steepslope");
        assert_eq!(canonical_layer_id("flood-history"), "floodhistory");
        assert_eq!(canonical_layer_id("landprice"), "landprice");
        assert_eq!(canonical_layer_id("schools"), "schools");
    }
}
