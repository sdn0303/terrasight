use crate::domain::error::DomainError;

/// Human-readable name for an administrative area.
///
/// Rejects empty / whitespace-only strings via [`DomainError::Validation`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AreaName(String);

impl AreaName {
    pub fn parse(s: &str) -> Result<Self, DomainError> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(DomainError::Validation(
                "area name must be non-empty".into(),
            ));
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Land price per square meter, stored in JPY (integer yen).
///
/// Rejects negative values via [`DomainError::Validation`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PricePerSqm(i64);

impl PricePerSqm {
    pub fn new(value: i64) -> Result<Self, DomainError> {
        if value < 0 {
            return Err(DomainError::Validation(format!(
                "price_per_sqm must be non-negative, got {value}"
            )));
        }
        Ok(Self(value))
    }

    pub fn value(self) -> i64 {
        self.0
    }
}

/// Building Coverage Ratio (建蔽率) as an integer percentage in `0..=100`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuildingCoverageRatio(i32);

impl BuildingCoverageRatio {
    pub fn new(value: i32) -> Result<Self, DomainError> {
        if !(0..=100).contains(&value) {
            return Err(DomainError::Validation(format!(
                "building coverage ratio must be in 0..=100, got {value}"
            )));
        }
        Ok(Self(value))
    }

    pub fn value(self) -> i32 {
        self.0
    }
}

/// Floor Area Ratio (容積率) as an integer percentage in `0..=2000`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FloorAreaRatio(i32);

impl FloorAreaRatio {
    pub fn new(value: i32) -> Result<Self, DomainError> {
        if !(0..=2000).contains(&value) {
            return Err(DomainError::Validation(format!(
                "floor area ratio must be in 0..=2000, got {value}"
            )));
        }
        Ok(Self(value))
    }

    pub fn value(self) -> i32 {
        self.0
    }
}

/// Distance in meters (non-negative by construction).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Meters(u32);

impl Meters {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn value(self) -> u32 {
        self.0
    }
}

/// Percentage value as an `f64` (domain convention: not clamped to `0..=100`;
/// negative values represent e.g. year-over-year decreases).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Percent(f64);

impl Percent {
    pub const fn zero() -> Self {
        Self(0.0)
    }

    pub fn new(value: f64) -> Self {
        Self(value)
    }

    pub fn value(self) -> f64 {
        self.0
    }
}

/// Zoning code (e.g. "YOYOKU_CODE_01"). Rejects empty strings.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ZoneCode(String);

impl ZoneCode {
    pub fn parse(s: &str) -> Result<Self, DomainError> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(DomainError::Validation(
                "zone code must be non-empty".into(),
            ));
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Record count, clamped to `>= 0` at construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RecordCount(i64);

impl RecordCount {
    pub fn new(value: i64) -> Self {
        Self(value.max(0))
    }

    pub fn value(self) -> i64 {
        self.0
    }
}

/// Postal address string for an observation point or entity.
///
/// Rejects empty / whitespace-only strings via [`DomainError::Validation`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Address(String);

impl Address {
    pub fn parse(s: &str) -> Result<Self, DomainError> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(DomainError::Validation("address must be non-empty".into()));
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// GeoJSON Feature in domain representation.
///
/// Corresponds to PostGIS `ST_AsGeoJSON` output. Coordinates follow
/// RFC 7946 `[longitude, latitude]` order.
///
/// Note: `serde_json::Value` is an allowed dependency in Domain layer
/// (data-representation library, not an I/O framework).
#[derive(Debug, Clone)]
pub struct GeoFeature {
    pub geometry: GeoJsonGeometry,
    pub properties: serde_json::Value,
}

/// GeoJSON geometry (flexible via `serde_json::Value` for coordinates).
#[derive(Debug, Clone)]
pub struct GeoJsonGeometry {
    pub r#type: String,
    pub coordinates: serde_json::Value,
}

/// Land price record for scoring and trend calculations.
#[derive(Debug, Clone)]
pub struct PriceRecord {
    pub year: i32,
    pub price_per_sqm: i64,
    /// Nearest observation point address (used in future score detail response).
    #[allow(dead_code)]
    pub address: String,
    /// Distance in meters to the observation point (used in future score detail response).
    #[allow(dead_code)]
    pub distance_m: f64,
}

/// Single data point in a price trend time series.
#[derive(Debug, Clone)]
pub struct TrendPoint {
    pub year: i32,
    pub price_per_sqm: i64,
}

/// Nearest observation point metadata for trend data.
#[derive(Debug, Clone)]
pub struct TrendLocation {
    pub address: String,
    pub distance_m: f64,
}

/// Aggregated land price statistics within a bounding box.
#[derive(Debug, Clone)]
pub struct LandPriceStats {
    pub avg_per_sqm: Option<f64>,
    pub median_per_sqm: Option<f64>,
    pub min_per_sqm: Option<i64>,
    pub max_per_sqm: Option<i64>,
    pub count: i64,
}

/// Risk statistics within a bounding box.
#[derive(Debug, Clone)]
pub struct RiskStats {
    pub flood_area_ratio: f64,
    pub steep_slope_area_ratio: f64,
    pub composite_risk: f64,
}

/// Facility counts within a bounding box.
#[derive(Debug, Clone)]
pub struct FacilityStats {
    pub schools: i64,
    pub medical: i64,
}

/// Aggregated area statistics (P0 fix: moved from Usecase to Domain).
#[derive(Debug, Clone)]
pub struct AreaStats {
    pub land_price: LandPriceStats,
    pub risk: RiskStats,
    pub facilities: FacilityStats,
    pub zoning_distribution: std::collections::HashMap<String, f64>,
}

/// Result of a per-layer bbox query with truncation metadata.
///
/// When the database returns more rows than `limit`, the repository fetches
/// `limit + 1` rows (N+1 pattern), sets `truncated = true`, and returns
/// only the first `limit` features. Callers can surface the `truncated` flag
/// and `limit` to MapLibre GL clients so they know to zoom in for full data.
#[derive(Debug, Clone)]
pub struct LayerResult {
    /// GeoJSON features returned (at most `limit` items).
    pub features: Vec<GeoFeature>,
    /// `true` when the result set was capped at `limit`.
    pub truncated: bool,
    /// The limit that was applied for this layer + zoom combination.
    pub limit: i64,
}

/// Health check result (P0 fix: moved from Usecase to Domain).
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub status: &'static str,
    pub db_connected: bool,
    pub reinfolib_key_set: bool,
    pub version: &'static str,
}

/// School accessibility details for TLS scoring.
#[derive(Debug, Clone)]
pub struct SchoolStats {
    pub count_800m: i64,
    pub has_primary: bool,
    pub has_junior_high: bool,
}

/// Medical facility details for TLS scoring.
#[derive(Debug, Clone)]
pub struct MedicalStats {
    pub hospital_count: i64,
    pub clinic_count: i64,
    pub total_beds: i64,
}

/// Z-score of ㎡ price within the same zoning type.
#[derive(Debug, Clone)]
pub struct ZScoreResult {
    pub z_score: f64,
    pub zone_type: String,
    pub sample_count: i64,
}

/// Aggregated statistics for an administrative area (prefecture or municipality).
///
/// `level` is either `"prefecture"` (2-digit code) or `"municipality"` (5-digit code).
/// `name` is a placeholder until the `admin_boundaries` table is populated by the
/// Phase 5 data pipeline — callers should treat it as informational only for now.
#[derive(Debug, Clone)]
pub struct AdminAreaStats {
    pub code: String,
    pub name: String,
    pub level: String,
    pub land_price: LandPriceStats,
    pub risk: RiskStats,
    pub facilities: FacilityStats,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_per_sqm_rejects_negative() {
        assert!(PricePerSqm::new(-1).is_err());
        assert_eq!(PricePerSqm::new(0).unwrap().value(), 0);
        assert_eq!(PricePerSqm::new(1_500_000).unwrap().value(), 1_500_000);
    }

    #[test]
    fn bcr_and_far_bounds() {
        assert!(BuildingCoverageRatio::new(-1).is_err());
        assert!(BuildingCoverageRatio::new(101).is_err());
        assert_eq!(BuildingCoverageRatio::new(60).unwrap().value(), 60);
        assert!(FloorAreaRatio::new(-1).is_err());
        assert!(FloorAreaRatio::new(2001).is_err());
        assert_eq!(FloorAreaRatio::new(400).unwrap().value(), 400);
    }

    #[test]
    fn meters_and_percent_constructors() {
        assert_eq!(Meters::new(500).value(), 500);
        assert_eq!(Percent::zero().value(), 0.0);
        assert_eq!(Percent::new(-5.2).value(), -5.2);
    }

    #[test]
    fn zone_code_rejects_empty() {
        assert!(ZoneCode::parse("").is_err());
        assert!(ZoneCode::parse("   ").is_err());
        assert_eq!(ZoneCode::parse(" Y1 ").unwrap().as_str(), "Y1");
    }

    #[test]
    fn record_count_clamps_negative_to_zero() {
        assert_eq!(RecordCount::new(-5).value(), 0);
        assert_eq!(RecordCount::new(0).value(), 0);
        assert_eq!(RecordCount::new(42).value(), 42);
    }

    #[test]
    fn area_name_accepts_nonempty_and_trims() {
        let n = AreaName::parse("  Shinjuku  ").unwrap();
        assert_eq!(n.as_str(), "Shinjuku");
    }

    #[test]
    fn area_name_rejects_empty_and_whitespace() {
        assert!(AreaName::parse("").is_err());
        assert!(AreaName::parse("   ").is_err());
        assert!(AreaName::parse("\t\n").is_err());
    }

    #[test]
    fn address_accepts_nonempty_and_trims() {
        let a = Address::parse("1-1 Shinjuku").unwrap();
        assert_eq!(a.as_str(), "1-1 Shinjuku");
    }

    #[test]
    fn address_rejects_empty() {
        assert!(Address::parse("").is_err());
        assert!(Address::parse("   ").is_err());
    }
}
