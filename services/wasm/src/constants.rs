//! Shared constants for layer IDs, JSON property keys, and GeoJSON format strings.
//!
//! All magic numbers and string literals referenced in more than one
//! location are collected here to satisfy `proj-no-magic-numbers`.

// ── Layer IDs ──
// Used in spatial_index.rs (extract_stats_data) and lib.rs (compute_area_stats).
// hyphen-case = FGB/WASM canonical form.

/// Canonical layer ID for the land price (公示地価) dataset.
pub(crate) const LAYER_LANDPRICE: &str = "landprice";
/// Canonical layer ID for the flood inundation history dataset.
pub(crate) const LAYER_FLOOD_HISTORY: &str = "flood-history";
/// Canonical layer ID for the flood hazard zone dataset.
pub(crate) const LAYER_FLOOD: &str = "flood";
/// Canonical layer ID for the steep slope hazard dataset (hyphen form).
pub(crate) const LAYER_STEEP_SLOPE: &str = "steep-slope";
/// Canonical layer ID for the steep slope hazard dataset (underscore form).
///
/// Some pipeline stages emit `steep_slope` instead of `steep-slope`; both are
/// accepted by [`crate::SpatialEngine::load_layer`] and map to the same index.
pub(crate) const LAYER_STEEP_SLOPE_ALT: &str = "steep_slope";
/// Canonical layer ID for the schools facility dataset.
pub(crate) const LAYER_SCHOOLS: &str = "schools";
/// Canonical layer ID for the medical facility dataset.
pub(crate) const LAYER_MEDICAL: &str = "medical";
/// Canonical layer ID for the railway network dataset.
pub(crate) const LAYER_RAILWAY: &str = "railway";
/// Canonical layer ID for the railway station dataset.
pub(crate) const LAYER_STATION: &str = "station";
/// Canonical layer ID for the zoning classification dataset.
pub(crate) const LAYER_ZONING: &str = "zoning";

// ── GeoJSON property keys ──
// Used when extracting feature properties in spatial_index.rs and stats.rs.

/// GeoJSON feature property key for the land price value (yen/m²).
pub(crate) const PROP_PRICE_PER_SQM: &str = "price_per_sqm";
/// GeoJSON feature property key for the zoning classification string.
pub(crate) const PROP_ZONE_TYPE: &str = "zone_type";

// ── GeoJSON structural keys ──
// Used in fgb_reader.rs for parsing.

/// Top-level `"type"` key present in all GeoJSON objects.
pub(crate) const GEOJSON_KEY_TYPE: &str = "type";
/// Key for the geometry object within a GeoJSON Feature.
pub(crate) const GEOJSON_KEY_GEOMETRY: &str = "geometry";
/// Key for the properties object within a GeoJSON Feature.
pub(crate) const GEOJSON_KEY_PROPERTIES: &str = "properties";
/// Key for the coordinates array within a GeoJSON geometry object.
pub(crate) const GEOJSON_KEY_COORDINATES: &str = "coordinates";
/// Key for the geometries array within a GeoJSON GeometryCollection.
pub(crate) const GEOJSON_KEY_GEOMETRIES: &str = "geometries";

// ── GeoJSON geometry type names ──

/// GeoJSON geometry type string for a single point.
pub(crate) const GEOJSON_TYPE_POINT: &str = "Point";
/// GeoJSON geometry type string for a line string.
pub(crate) const GEOJSON_TYPE_LINE_STRING: &str = "LineString";
/// GeoJSON geometry type string for a polygon (with optional holes).
pub(crate) const GEOJSON_TYPE_POLYGON: &str = "Polygon";
/// GeoJSON geometry type string for a collection of polygons.
pub(crate) const GEOJSON_TYPE_MULTI_POLYGON: &str = "MultiPolygon";

// ── GeoJSON FeatureCollection format ──

/// Opening fragment for a serialised GeoJSON FeatureCollection.
///
/// Concatenate [`FC_HEADER`], comma-separated Feature strings, then [`FC_FOOTER`]
/// to build a complete `FeatureCollection` without a `serde_json` allocation.
pub(crate) const FC_HEADER: &str = r#"{"type":"FeatureCollection","features":["#;
/// Closing fragment for a serialised GeoJSON FeatureCollection.
pub(crate) const FC_FOOTER: &str = "]}";

// ── Coordinate validation ──

/// Minimum number of elements required in a GeoJSON coordinate array (`[x, y, ...]`).
///
/// RFC 7946 requires at least a longitude and latitude component.
pub(crate) const MIN_COORD_PAIR_LEN: usize = 2;

// ── WGS84 coordinate system limits ──

/// Minimum valid latitude in the WGS84 coordinate system (degrees).
pub(crate) const LAT_MIN: f64 = -90.0;
/// Maximum valid latitude in the WGS84 coordinate system (degrees).
pub(crate) const LAT_MAX: f64 = 90.0;
/// Minimum valid longitude in the WGS84 coordinate system (degrees).
pub(crate) const LNG_MIN: f64 = -180.0;
/// Maximum valid longitude in the WGS84 coordinate system (degrees).
pub(crate) const LNG_MAX: f64 = 180.0;

/// Inclusive latitude range `[-90.0, 90.0]` for range-check expressions.
pub(crate) const LAT_RANGE: std::ops::RangeInclusive<f64> = LAT_MIN..=LAT_MAX;
/// Inclusive longitude range `[-180.0, 180.0]` for range-check expressions.
pub(crate) const LNG_RANGE: std::ops::RangeInclusive<f64> = LNG_MIN..=LNG_MAX;

// ── Risk computation weights ──

/// Weight applied to the flood area ratio when computing composite risk.
///
/// `composite_risk = RISK_WEIGHT_FLOOD * flood_ratio + RISK_WEIGHT_STEEP * steep_ratio`.
pub(crate) const RISK_WEIGHT_FLOOD: f64 = 0.6;
/// Weight applied to the steep slope area ratio when computing composite risk.
pub(crate) const RISK_WEIGHT_STEEP: f64 = 0.4;

// ── Zoning classification keywords ──

/// Substring present in all commercial zone type strings (e.g. `"商業地域"`).
///
/// Used by the TLS zoning score computation to identify commercial zones
/// from the `zone_type` GeoJSON property.
pub(crate) const COMMERCIAL_ZONE_KEYWORD: &str = "商業";

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
