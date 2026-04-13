/// Calculate bounding box area in square degrees (approximate).
///
/// Used for metrics tracking (`spatial.bbox.area_deg2`).
///
/// # Examples
///
/// ```
/// use terrasight_geo::spatial::bbox_area_deg2;
///
/// let area = bbox_area_deg2(35.65, 139.70, 35.70, 139.80);
/// assert!((area - 0.005).abs() < 1e-9);
///
/// assert_eq!(bbox_area_deg2(0.0, 0.0, 1.0, 1.0), 1.0);
/// ```
pub fn bbox_area_deg2(south: f64, west: f64, north: f64, east: f64) -> f64 {
    (north - south).abs() * (east - west).abs()
}

/// Buffer size in degrees (~15m at Tokyo latitude 35.68°).
pub const BUFFER_DEG: f64 = 0.00015;

const MAX_FEATURES: i64 = 10_000;

// ── Layer density (features per square degree at zoom 14) ──
const DENSITY_LANDPRICE: f64 = 50_000.0;
const DENSITY_FLOOD: f64 = 150_000.0;
const DENSITY_ZONING: f64 = 100_000.0;
const DENSITY_STEEP_SLOPE: f64 = 80_000.0;
const DENSITY_SCHOOLS: f64 = 30_000.0;
const DENSITY_MEDICAL: f64 = 80_000.0;
const DENSITY_DEFAULT: f64 = 30_000.0;

// ── Feature limit constants ──
const LOW_ZOOM_THRESHOLD: u32 = 10;
const LOW_ZOOM_DIVISOR: i64 = 4;

// ── Layer name constants ──
const LAYER_LANDPRICE: &str = "landprice";
const LAYER_FLOOD: &str = "flood";
const LAYER_ZONING: &str = "zoning";
const LAYER_STEEP_SLOPE: &str = "steep_slope";
const LAYER_SCHOOLS: &str = "schools";
const LAYER_MEDICAL: &str = "medical";

fn layer_density(layer: &str) -> f64 {
    match layer {
        LAYER_LANDPRICE => DENSITY_LANDPRICE,
        LAYER_FLOOD => DENSITY_FLOOD,
        LAYER_ZONING => DENSITY_ZONING,
        LAYER_STEEP_SLOPE => DENSITY_STEEP_SLOPE,
        LAYER_SCHOOLS => DENSITY_SCHOOLS,
        LAYER_MEDICAL => DENSITY_MEDICAL,
        _ => DENSITY_DEFAULT,
    }
}

/// Compute the feature limit for a given layer, bounding-box area, and zoom level.
///
/// Formula: `min(ceil(bbox_area × density), 10_000)`. If zoom < 10, the result
/// is divided by 4. The minimum returned value is 1.
///
/// # Examples
///
/// ```
/// use terrasight_geo::spatial::compute_feature_limit;
///
/// assert_eq!(compute_feature_limit("flood", 0.02, 12), 3_000);
/// assert_eq!(compute_feature_limit("flood", 1.0, 12), 10_000);
/// ```
pub fn compute_feature_limit(layer: &str, bbox_area_deg2: f64, zoom: u32) -> i64 {
    let density = layer_density(layer);
    let raw = (bbox_area_deg2 * density).ceil() as i64;
    let capped = raw.min(MAX_FEATURES);
    let adjusted = if zoom < LOW_ZOOM_THRESHOLD {
        capped / LOW_ZOOM_DIVISOR
    } else {
        capped
    };
    adjusted.max(1)
}

/// Create a closed polygon ring of 5 vertices forming a ~30m × 30m square
/// around the given point.
///
/// Vertices are in counter-clockwise order: SW → SE → NE → NW → SW (close).
/// Coordinates follow RFC 7946: `[longitude, latitude]`.
///
/// # Examples
///
/// ```
/// use terrasight_geo::spatial::{point_to_polygon, BUFFER_DEG};
///
/// let ring = point_to_polygon(139.7, 35.68);
/// assert_eq!(ring[0], ring[4]);
/// ```
pub fn point_to_polygon(lng: f64, lat: f64) -> [[f64; 2]; 5] {
    let diameter = 2.0 * BUFFER_DEG;
    let w = lng - BUFFER_DEG;
    let e = w + diameter; // ensures e - w == 2.0 * BUFFER_DEG exactly
    let s = lat - BUFFER_DEG;
    let n = s + diameter; // ensures n - s == 2.0 * BUFFER_DEG exactly

    [
        [w, s], // SW
        [e, s], // SE
        [e, n], // NE
        [w, n], // NW
        [w, s], // SW close
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bbox_tokyo_area() {
        let area = bbox_area_deg2(35.65, 139.70, 35.70, 139.80);
        assert!((area - 0.005).abs() < 1e-9, "expected ~0.005, got {area}");
    }

    #[test]
    fn bbox_unit_square() {
        assert_eq!(bbox_area_deg2(0.0, 0.0, 1.0, 1.0), 1.0);
    }

    #[test]
    fn bbox_inverted_coordinates_still_positive() {
        // south/north and west/east swapped — abs() ensures positive area
        let normal = bbox_area_deg2(35.65, 139.70, 35.70, 139.80);
        let inverted = bbox_area_deg2(35.70, 139.80, 35.65, 139.70);
        assert!((normal - inverted).abs() < f64::EPSILON);
    }

    #[test]
    fn bbox_zero_area_point() {
        assert_eq!(bbox_area_deg2(35.0, 139.0, 35.0, 139.0), 0.0);
    }

    // --- compute_feature_limit tests ---

    #[test]
    fn feature_limit_small_bbox_flood() {
        // 0.02 deg² × 150_000 = 3_000
        assert_eq!(compute_feature_limit("flood", 0.02, 12), 3_000);
    }

    #[test]
    fn feature_limit_caps_at_max() {
        // 1.0 × 150_000 = 150_000 → capped at 10_000
        assert_eq!(compute_feature_limit("flood", 1.0, 12), 10_000);
    }

    #[test]
    fn feature_limit_low_zoom_divides() {
        // 0.5 × 150_000 = 75_000 → cap 10_000 → ÷4 = 2_500
        assert_eq!(compute_feature_limit("flood", 0.5, 8), 2_500);
    }

    #[test]
    fn feature_limit_unknown_layer_uses_default() {
        assert!(compute_feature_limit("unknown", 0.01, 12) > 0);
    }

    // --- point_to_polygon tests ---

    #[test]
    fn point_to_polygon_creates_closed_ring() {
        let ring = point_to_polygon(139.7, 35.68);
        assert_eq!(ring.len(), 5);
        assert_eq!(ring[0], ring[4]);
    }

    #[test]
    fn point_to_polygon_buffer_size() {
        let ring = point_to_polygon(139.7, 35.68);
        let width = ring[1][0] - ring[0][0];
        // f64 subtraction of two values ~139.7 introduces rounding error on the
        // order of 1 ULP(139.7) ≈ 2.8e-14. Using 1e-10 as absolute tolerance
        // is sufficient to confirm the width equals 2 × BUFFER_DEG.
        assert!(
            (width - 2.0 * BUFFER_DEG).abs() < 1e-10,
            "expected width {}, got {width}",
            2.0 * BUFFER_DEG
        );
    }
}
