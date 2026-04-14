//! Spatial query support functions for bounding-box filtering and feature-count limits.
//!
//! These utilities sit between the HTTP handler layer (which parses raw bbox
//! parameters) and the database layer (which executes the query). They answer
//! two questions:
//!
//! 1. **How large is the requested area?** — [`bbox_area_deg2`] returns the
//!    approximate area in square degrees for metrics and limit calculations.
//! 2. **How many features should the query return?** — [`compute_feature_limit`]
//!    caps the row count per layer type and zoom level, preventing runaway
//!    queries over dense datasets such as [`LayerKind::Flood`].

use crate::coord::GeoBBox;
use crate::coord::GeoCoord;

/// Calculate bounding box area in square degrees (approximate).
///
/// Used for metrics tracking (`spatial.bbox.area_deg2`).
///
/// # Examples
///
/// ```
/// use terrasight_geo::coord::GeoBBox;
/// use terrasight_geo::spatial::bbox_area_deg2;
///
/// let bbox = GeoBBox { south: 35.65, west: 139.70, north: 35.70, east: 139.80 };
/// let area = bbox_area_deg2(&bbox);
/// assert!((area - 0.005).abs() < 1e-9);
///
/// let unit = GeoBBox { south: 0.0, west: 0.0, north: 1.0, east: 1.0 };
/// assert_eq!(bbox_area_deg2(&unit), 1.0);
/// ```
#[must_use]
pub fn bbox_area_deg2(bbox: &GeoBBox) -> f64 {
    ((bbox.north - bbox.south) * (bbox.east - bbox.west)).abs()
}

/// Buffer size in degrees used by [`point_to_polygon`] (~15 m at Tokyo latitude 35.68°).
pub const POINT_TO_POLYGON_BUFFER_DEG: f64 = 0.00015;

// Hard cap on the number of features returned by any single query.
const MAX_FEATURES: i64 = 10_000;
// Zoom levels below this threshold are coarser; the feature limit is reduced.
const LOW_ZOOM_THRESHOLD: u8 = 10;
// Divisor applied to the feature limit when zoom < LOW_ZOOM_THRESHOLD.
const LOW_ZOOM_DIVISOR: i64 = 4;

// ── Layer density (features per square degree at zoom 14) ──
const DENSITY_LANDPRICE: f64 = 50_000.0;
const DENSITY_FLOOD: f64 = 150_000.0;
const DENSITY_ZONING: f64 = 100_000.0;
const DENSITY_STEEP_SLOPE: f64 = 80_000.0;
const DENSITY_SCHOOLS: f64 = 30_000.0;
const DENSITY_MEDICAL: f64 = 80_000.0;
const DENSITY_DEFAULT: f64 = 30_000.0;

/// Typed enum identifying the GIS data layers served by the backend.
///
/// Using an enum instead of a raw `&str` at call sites encodes the set of
/// valid layer names in the type system and lets [`compute_feature_limit`]
/// dispatch on density without string comparison.
///
/// Each variant carries an implicit *features-per-square-degree* density used
/// to estimate query result size at zoom 14. Denser layers (e.g. [`LayerKind::Flood`])
/// reach the hard cap sooner than sparse ones (e.g. [`LayerKind::Schools`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerKind {
    /// Published land price survey points (地価公示).
    ///
    /// Moderate density (~50 000 points/deg²). Points are irregularly spaced
    /// and concentrated in urban areas.
    LandPrice,
    /// Flood hazard zone polygons (洪水浸水想定区域).
    ///
    /// Highest density (~150 000 features/deg²). Fine-grained polygon meshes
    /// covering river basins reach the feature cap even at small bounding boxes.
    Flood,
    /// Urban planning use-zone polygons (用途地域).
    ///
    /// High density (~100 000 features/deg²). Covers all designated urban areas
    /// in Japan with polygon boundaries delineating zoning categories.
    Zoning,
    /// Steep slope disaster-risk zones (急傾斜地崩壊危険区域).
    ///
    /// Moderate-high density (~80 000 features/deg²). Polygons are concentrated
    /// in mountainous and hilly terrain; sparse in flat urban plains.
    SteepSlope,
    /// Public school locations (学校).
    ///
    /// Low density (~30 000 points/deg²). Point features; density scales
    /// linearly with residential population.
    Schools,
    /// Medical facility locations (医療施設).
    ///
    /// Moderate-high density (~80 000 points/deg²). Includes hospitals,
    /// clinics, and pharmacies; dense in commercial and mixed-use areas.
    Medical,
    /// Fallback for any layer not explicitly enumerated.
    ///
    /// Uses the default density (~30 000 features/deg²), which matches the
    /// conservative lower bound of the known layers.
    Other,
}

impl LayerKind {
    fn density(self) -> f64 {
        match self {
            Self::LandPrice => DENSITY_LANDPRICE,
            Self::Flood => DENSITY_FLOOD,
            Self::Zoning => DENSITY_ZONING,
            Self::SteepSlope => DENSITY_STEEP_SLOPE,
            Self::Schools => DENSITY_SCHOOLS,
            Self::Medical => DENSITY_MEDICAL,
            Self::Other => DENSITY_DEFAULT,
        }
    }
}

/// Compute the feature limit for a given layer, bounding-box area, and zoom level.
///
/// Formula: `min(ceil(bbox_area × density), 10_000)`. If zoom < 10, the result
/// is divided by 4. The minimum returned value is 1.
///
/// The `zoom` parameter is typed as `u8` because Web Mercator zoom levels are
/// defined in the range 0–22 and never exceed 255.
///
/// # Examples
///
/// ```
/// use terrasight_geo::spatial::{LayerKind, compute_feature_limit};
///
/// assert_eq!(compute_feature_limit(LayerKind::Flood, 0.02, 12), 3_000);
/// assert_eq!(compute_feature_limit(LayerKind::Flood, 1.0, 12), 10_000);
/// ```
#[must_use]
pub fn compute_feature_limit(layer: LayerKind, bbox_area_deg2: f64, zoom: u8) -> i64 {
    let raw = (bbox_area_deg2 * layer.density()).ceil() as i64;
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
/// use terrasight_geo::coord::GeoCoord;
/// use terrasight_geo::spatial::{point_to_polygon, POINT_TO_POLYGON_BUFFER_DEG};
///
/// let coord = GeoCoord { lng: 139.7, lat: 35.68 };
/// let ring = point_to_polygon(&coord);
/// assert_eq!(ring[0], ring[4]);
/// ```
#[must_use]
pub fn point_to_polygon(coord: &GeoCoord) -> [[f64; 2]; 5] {
    let diameter = 2.0 * POINT_TO_POLYGON_BUFFER_DEG;
    let w = coord.lng - POINT_TO_POLYGON_BUFFER_DEG;
    let e = w + diameter; // ensures e - w == 2.0 * POINT_TO_POLYGON_BUFFER_DEG exactly
    let s = coord.lat - POINT_TO_POLYGON_BUFFER_DEG;
    let n = s + diameter; // ensures n - s == 2.0 * POINT_TO_POLYGON_BUFFER_DEG exactly

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
        let bbox = GeoBBox {
            south: 35.65,
            west: 139.70,
            north: 35.70,
            east: 139.80,
        };
        let area = bbox_area_deg2(&bbox);
        assert!((area - 0.005).abs() < 1e-9, "expected ~0.005, got {area}");
    }

    #[test]
    fn bbox_unit_square() {
        let bbox = GeoBBox {
            south: 0.0,
            west: 0.0,
            north: 1.0,
            east: 1.0,
        };
        assert_eq!(bbox_area_deg2(&bbox), 1.0);
    }

    #[test]
    fn bbox_inverted_coordinates_still_positive() {
        // south/north and west/east swapped — abs() ensures positive area
        let normal = bbox_area_deg2(&GeoBBox {
            south: 35.65,
            west: 139.70,
            north: 35.70,
            east: 139.80,
        });
        let inverted = bbox_area_deg2(&GeoBBox {
            south: 35.70,
            west: 139.80,
            north: 35.65,
            east: 139.70,
        });
        assert!((normal - inverted).abs() < f64::EPSILON);
    }

    #[test]
    fn bbox_zero_area_point() {
        let bbox = GeoBBox {
            south: 35.0,
            west: 139.0,
            north: 35.0,
            east: 139.0,
        };
        assert_eq!(bbox_area_deg2(&bbox), 0.0);
    }

    // --- compute_feature_limit tests ---

    #[test]
    fn feature_limit_small_bbox_flood() {
        // 0.02 deg² × 150_000 = 3_000
        assert_eq!(compute_feature_limit(LayerKind::Flood, 0.02, 12), 3_000);
    }

    #[test]
    fn feature_limit_caps_at_max() {
        // 1.0 × 150_000 = 150_000 → capped at 10_000
        assert_eq!(compute_feature_limit(LayerKind::Flood, 1.0, 12), 10_000);
    }

    #[test]
    fn feature_limit_low_zoom_divides() {
        // 0.5 × 150_000 = 75_000 → cap 10_000 → ÷4 = 2_500
        assert_eq!(compute_feature_limit(LayerKind::Flood, 0.5, 8), 2_500);
    }

    #[test]
    fn feature_limit_other_layer_uses_default() {
        assert!(compute_feature_limit(LayerKind::Other, 0.01, 12) > 0);
    }

    // --- point_to_polygon tests ---

    #[test]
    fn point_to_polygon_creates_closed_ring() {
        let coord = GeoCoord {
            lng: 139.7,
            lat: 35.68,
        };
        let ring = point_to_polygon(&coord);
        assert_eq!(ring.len(), 5);
        assert_eq!(ring[0], ring[4]);
    }

    #[test]
    fn point_to_polygon_buffer_size() {
        let coord = GeoCoord {
            lng: 139.7,
            lat: 35.68,
        };
        let ring = point_to_polygon(&coord);
        let width = ring[1][0] - ring[0][0];
        // f64 subtraction of two values ~139.7 introduces rounding error on the
        // order of 1 ULP(139.7) ≈ 2.8e-14. Using 1e-10 as absolute tolerance
        // is sufficient to confirm the width equals 2 × POINT_TO_POLYGON_BUFFER_DEG.
        assert!(
            (width - 2.0 * POINT_TO_POLYGON_BUFFER_DEG).abs() < 1e-10,
            "expected width {}, got {width}",
            2.0 * POINT_TO_POLYGON_BUFFER_DEG
        );
    }
}
