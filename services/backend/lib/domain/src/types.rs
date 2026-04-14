//! Aggregated statistics DTOs shared by the Backend and WASM runtimes.
//!
//! These types are serialised directly to JSON for the `/api/stats` endpoint
//! and consumed by the MapLibre GL sidebar panel. Both runtimes share these
//! structs to guarantee that the wire format never diverges.

use serde::Serialize;

/// Aggregated land price statistics within a bounding box.
///
/// Produced by the stats use-case after querying the `land_prices` PostGIS
/// table for all points within the current map viewport. All price fields are
/// expressed in **円 per square metre (円/㎡)**.
///
/// Fields are `Option` because a viewport may contain no land-price data points
/// (e.g. a remote area or ocean), in which case the backend returns a zeroed
/// struct rather than a JSON `null`.
///
/// # Examples
///
/// ```
/// use terrasight_domain::types::LandPriceStats;
///
/// let stats = LandPriceStats::default();
/// assert_eq!(stats.count, 0);
/// assert!(stats.avg_per_sqm.is_none());
/// ```
#[derive(Debug, Clone, Default, Serialize)]
pub struct LandPriceStats {
    /// Mean price per square metre across all sampled points, or `None` when
    /// no data is available in the bounding box.
    pub avg_per_sqm: Option<f64>,

    /// Median price per square metre, or `None` when no data is available.
    ///
    /// The median is more robust than the mean in areas with extreme outliers
    /// (e.g. high-rise commercial plots adjacent to residential land).
    pub median_per_sqm: Option<f64>,

    /// Minimum observed price per square metre, or `None` when no data is available.
    pub min_per_sqm: Option<i64>,

    /// Maximum observed price per square metre, or `None` when no data is available.
    pub max_per_sqm: Option<i64>,

    /// Total number of land-price data points within the bounding box.
    ///
    /// A value of `0` means no data is available; all `Option` fields will be
    /// `None` in that case.
    pub count: i64,
}

/// Aggregated hazard area statistics within a bounding box.
///
/// Produced by the stats use-case after intersecting flood and steep-slope
/// hazard polygons with the viewport. Used to populate the risk panel in the
/// frontend sidebar and to compute the S1 disaster axis confidence.
///
/// The `composite_risk` field is a weighted blend:
/// `0.6 × flood_area_ratio + 0.4 × steep_slope_area_ratio`
/// (see [`crate::constants::STATS_RISK_WEIGHT_FLOOD`] and
/// [`crate::constants::STATS_RISK_WEIGHT_STEEP`]).
#[derive(Debug, Clone, Serialize)]
pub struct RiskStats {
    /// Fraction of the bounding-box area covered by designated flood hazard
    /// zones, in the range `[0.0, 1.0]`.
    pub flood_area_ratio: f64,

    /// Fraction of the bounding-box area covered by steep-slope (急傾斜地)
    /// hazard zones, in the range `[0.0, 1.0]`.
    pub steep_slope_area_ratio: f64,

    /// Weighted composite risk index in the range `[0.0, 1.0]`.
    ///
    /// Higher values indicate greater overall hazard exposure.
    /// Computed as `0.6 × flood_area_ratio + 0.4 × steep_slope_area_ratio`.
    pub composite_risk: f64,
}
