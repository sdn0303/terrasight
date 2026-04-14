//! Lightweight geographic coordinate types for pure-math functions.
//!
//! Unlike the validated `BBox` and `Coord` types in the domain layer, these
//! carry no invariants — they are simple data carriers that eliminate
//! parameter-ordering bugs in function signatures.
//!
//! # Design rationale
//!
//! Raw `f64` parameters like `bbox_area_deg2(south, west, north, east)` are
//! indistinguishable at the call site: passing arguments in the wrong order
//! silently produces incorrect results. [`GeoBBox`] and [`GeoCoord`] make the
//! intended field explicit via named struct construction.

/// Unvalidated geographic coordinate pair.
///
/// Field order follows GeoJSON convention: `[longitude, latitude]`.
///
/// # Examples
///
/// ```
/// use terrasight_geo::coord::GeoCoord;
///
/// let coord = GeoCoord { lng: 139.7671, lat: 35.6812 };
/// assert_eq!(coord.lng, 139.7671);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct GeoCoord {
    /// Longitude in WGS-84 decimal degrees.
    pub lng: f64,
    /// Latitude in WGS-84 decimal degrees.
    pub lat: f64,
}

/// Unvalidated geographic bounding box in WGS-84 decimal degrees.
///
/// Fields use compass directions for clarity, avoiding the ambiguity of
/// positional `f64` parameters. The canonical field order is
/// `(south, west, north, east)`.
///
/// # Examples
///
/// ```
/// use terrasight_geo::coord::GeoBBox;
///
/// let bbox = GeoBBox { south: 35.65, west: 139.70, north: 35.70, east: 139.80 };
/// assert!(bbox.north > bbox.south);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct GeoBBox {
    /// Southern latitude bound.
    pub south: f64,
    /// Western longitude bound.
    pub west: f64,
    /// Northern latitude bound.
    pub north: f64,
    /// Eastern longitude bound.
    pub east: f64,
}

impl GeoBBox {
    /// Construct from individual compass components.
    ///
    /// Use this when you already have the four values as separate variables
    /// (e.g., from domain `BBox` accessor methods) and prefer named
    /// construction over a bare struct literal for readability.
    ///
    /// # Examples
    ///
    /// ```
    /// use terrasight_geo::coord::GeoBBox;
    ///
    /// let bbox = GeoBBox::new(35.65, 139.70, 35.70, 139.80);
    /// assert_eq!(bbox.south, 35.65);
    /// assert_eq!(bbox.east, 139.80);
    /// ```
    #[must_use]
    pub fn new(south: f64, west: f64, north: f64, east: f64) -> Self {
        Self {
            south,
            west,
            north,
            east,
        }
    }
}
