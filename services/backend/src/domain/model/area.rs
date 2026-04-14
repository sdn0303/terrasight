//! Administrative area and facility statistics domain types.

use std::collections::HashMap;

pub use super::primitives::AreaName;
pub use terrasight_domain::types::RiskStats;

use super::primitives::AreaCode;
use crate::domain::model::price::LandPriceStats;

/// Facility counts within a bounding box.
///
/// Used in [`AreaStats`] and [`AdminAreaStats`] to give investors a quick
/// read on neighbourhood amenity density.
#[derive(Debug, Clone)]
pub struct FacilityStats {
    /// Number of school facilities (all levels) within the queried area.
    pub schools: i64,
    /// Number of medical facilities (hospitals + clinics) within the queried area.
    pub medical: i64,
}

/// Aggregated area statistics for the `/api/v1/stats` endpoint.
///
/// Moved to the domain layer (P0 polish) so both the usecase and handler can
/// reference the type without importing from each other.
#[derive(Debug, Clone)]
pub struct AreaStats {
    /// Aggregated land price statistics (min, max, average, median).
    pub land_price: LandPriceStats,
    /// Disaster risk statistics (flood and steep-slope coverage ratios).
    pub risk: RiskStats,
    /// Nearby facility counts.
    pub facilities: FacilityStats,
    /// Share of each zoning type within the bbox, as a fraction summing to 1.0.
    pub zoning_distribution: HashMap<String, f64>,
}

/// Aggregated statistics for an administrative area (prefecture or municipality).
///
/// `level` is either `"prefecture"` (2-digit code) or `"municipality"` (5-digit code).
/// `name` is a placeholder until the `admin_boundaries` table is populated by the
/// Phase 5 data pipeline — callers should treat it as informational only for now.
#[derive(Debug, Clone)]
pub struct AdminAreaStats {
    /// JIS area code: 2-digit prefecture or 5-digit municipality.
    pub code: AreaCode,
    /// Human-readable name for display (informational; see struct note).
    pub name: AreaName,
    /// Granularity string: `"prefecture"` or `"municipality"`.
    pub level: String,
    /// Aggregated land price statistics for the area.
    pub land_price: LandPriceStats,
    /// Disaster risk statistics for the area.
    pub risk: RiskStats,
    /// Facility counts within the area boundary.
    pub facilities: FacilityStats,
}
