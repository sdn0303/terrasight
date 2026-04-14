//! Core domain entities, shared value types, and the `nonempty_string_type!`
//! macro.
//!
//! This module defines the fundamental building blocks used across every layer
//! of the Terrasight backend. Types here carry business meaning: they are the
//! concepts that domain experts reason about (land prices, geographic features,
//! opportunity scores, etc.) rather than technical implementation details.
//!
//! ## Design notes
//!
//! - Types with validation constraints (e.g. [`PricePerSqm`], [`BuildingCoverageRatio`])
//!   reject invalid values at construction time so the rest of the codebase
//!   can treat them as always-valid.
//! - [`GeoFeature`] and [`GeoJsonGeometry`] wrap `serde_json::Value` for
//!   the coordinate payload. `serde_json` is permitted in the domain layer
//!   because it is a data-representation library, not an I/O framework.
//! - [`LandPriceStats`] and [`RiskStats`] are re-exported from the shared
//!   `terrasight-domain` crate so downstream usecases and handlers use a
//!   single canonical type.

use crate::domain::constants::{BCR_MAX, BCR_MIN, FAR_MAX, FAR_MIN};
use crate::domain::error::DomainError;
use crate::domain::value_object::{AreaCode, Coord, OpportunitySignal, RiskLevel, TlsScore, Year};

/// Generate a validated non-empty string newtype.
///
/// # Generated API
///
/// The macro expands to a `pub struct $Name(String)` with:
/// - `parse(s: &str) -> Result<Self, DomainError>` — trims whitespace and
///   rejects empty-after-trim inputs with [`DomainError::Validation`].
/// - `as_str(&self) -> &str` — borrows the inner string.
/// - `Display` — delegates to the inner `String`.
/// - `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`.
///
/// The generated type trims whitespace on construction and rejects strings that
/// are empty after trimming.  It also derives `Debug`, `Clone`, `PartialEq`,
/// `Eq`, `Hash`, and `Display`.
macro_rules! nonempty_string_type {
    ($Name:ident, $doc:expr, $err_msg:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $Name(String);

        impl $Name {
            /// Parse a string into a validated, whitespace-trimmed value.
            ///
            /// # Errors
            ///
            /// Returns [`DomainError::Validation`] if the input is empty after trimming.
            pub fn parse(s: &str) -> Result<Self, DomainError> {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    return Err(DomainError::Validation($err_msg.into()));
                }
                Ok(Self(trimmed.to_owned()))
            }

            /// Borrows the inner string slice.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $Name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

nonempty_string_type!(
    AreaName,
    "Human-readable area name (e.g. \"新宿区\", \"Shinjuku\"). Whitespace-trimmed and non-empty by construction.",
    "area name must be non-empty"
);

nonempty_string_type!(
    Address,
    "Postal or street address, trimmed and non-empty by construction. Used as the human-readable label for land price observation points.",
    "address must be non-empty"
);

nonempty_string_type!(
    ZoneCode,
    "Urban-planning zone code (用途地域コード), e.g. \"商業地域\". Trimmed and non-empty by construction. Code set defined by MLIT.",
    "zone code must be non-empty"
);

/// Land price per square meter, stored in JPY (integer yen).
///
/// Rejects negative values via [`DomainError::Validation`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PricePerSqm(i64);

impl PricePerSqm {
    /// Construct a [`PricePerSqm`] from a raw integer yen value.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Validation`] if `value` is negative.
    ///
    /// # Examples
    ///
    /// ```
    /// # use terrasight_api::domain::entity::PricePerSqm;
    /// let price = PricePerSqm::new(1_500_000)?;
    /// assert_eq!(price.value(), 1_500_000);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(value: i64) -> Result<Self, DomainError> {
        if value < 0 {
            return Err(DomainError::Validation(format!(
                "price_per_sqm must be non-negative, got {value}"
            )));
        }
        Ok(Self(value))
    }

    /// Return the raw integer yen value.
    pub fn value(self) -> i64 {
        self.0
    }
}

/// Building Coverage Ratio (建蔽率) as an integer percentage in `0..=100`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuildingCoverageRatio(i32);

impl BuildingCoverageRatio {
    /// Construct a [`BuildingCoverageRatio`] from a percentage integer.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Validation`] if `value` is outside `[BCR_MIN, BCR_MAX]`
    /// (currently `0..=100`).
    ///
    /// # Examples
    ///
    /// ```
    /// # use terrasight_api::domain::entity::BuildingCoverageRatio;
    /// let bcr = BuildingCoverageRatio::new(60)?;
    /// assert_eq!(bcr.value(), 60);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(value: i32) -> Result<Self, DomainError> {
        if !(BCR_MIN..=BCR_MAX).contains(&value) {
            return Err(DomainError::Validation(format!(
                "building coverage ratio must be in {BCR_MIN}..={BCR_MAX}, got {value}"
            )));
        }
        Ok(Self(value))
    }

    /// Return the raw integer percentage.
    pub fn value(self) -> i32 {
        self.0
    }
}

/// Floor Area Ratio (容積率) as an integer percentage in `0..=2000`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FloorAreaRatio(i32);

impl FloorAreaRatio {
    /// Construct a [`FloorAreaRatio`] from a percentage integer.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Validation`] if `value` is outside `[FAR_MIN, FAR_MAX]`
    /// (currently `0..=2000`).
    ///
    /// # Examples
    ///
    /// ```
    /// # use terrasight_api::domain::entity::FloorAreaRatio;
    /// let far = FloorAreaRatio::new(400)?;
    /// assert_eq!(far.value(), 400);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(value: i32) -> Result<Self, DomainError> {
        if !(FAR_MIN..=FAR_MAX).contains(&value) {
            return Err(DomainError::Validation(format!(
                "floor area ratio must be in {FAR_MIN}..={FAR_MAX}, got {value}"
            )));
        }
        Ok(Self(value))
    }

    /// Return the raw integer percentage.
    pub fn value(self) -> i32 {
        self.0
    }
}

/// Distance in meters (non-negative by construction).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Meters(u32);

impl Meters {
    /// Wrap a non-negative meter distance.
    ///
    /// Construction is infallible because `u32` cannot represent negative values.
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    /// Return the raw meter count.
    pub fn value(self) -> u32 {
        self.0
    }
}

/// Percentage value as an `f64` (domain convention: not clamped to `0..=100`;
/// negative values represent e.g. year-over-year decreases).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Percent(f64);

impl Percent {
    /// The zero percentage (no change).
    pub const fn zero() -> Self {
        Self(0.0)
    }

    /// Wrap any `f64` as a percentage.
    ///
    /// Negative values represent decreases (e.g. `-3.5` means −3.5% year-over-year).
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    /// Return the raw `f64` percentage value.
    pub fn value(self) -> f64 {
        self.0
    }
}

/// Record count, clamped to `>= 0` at construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RecordCount(i64);

impl RecordCount {
    /// Wrap a count, clamping negative inputs to zero.
    ///
    /// Database `COUNT(*)` calls may return `NULL` which SQLx maps to `0i64`,
    /// but defensive callers can also pass negative raw values safely.
    pub fn new(value: i64) -> Self {
        Self(value.max(0))
    }

    /// Return the non-negative record count.
    pub fn value(self) -> i64 {
        self.0
    }
}

/// GeoJSON Feature in domain representation.
///
/// Corresponds to PostGIS `ST_AsGeoJSON` output. Coordinates follow
/// RFC 7946 `[longitude, latitude]` order.
///
/// Note: `serde_json::Value` is an allowed dependency in the domain layer —
/// it is a data-representation library, not an I/O framework.
#[derive(Debug, Clone)]
pub struct GeoFeature {
    /// The GeoJSON geometry (type + coordinates).
    pub geometry: GeoJsonGeometry,
    /// Arbitrary feature properties serialized from the database row.
    pub properties: serde_json::Value,
}

/// GeoJSON geometry type identifier (RFC 7946 §3.1).
///
/// Encodes the set of valid GeoJSON geometry types as an enum so that
/// `GeoJsonGeometry` cannot carry an arbitrary or misspelled type string.
/// The `as_str` method returns the canonical RFC 7946 name; `from_db_str`
/// maps the PostGIS `ST_GeometryType` output to the corresponding variant.
#[derive(Debug, Clone, serde::Serialize)]
pub enum GeoJsonType {
    /// Coordinate pair geometry (RFC 7946 §3.1.2).
    Point,
    /// Closed-ring polygon geometry (RFC 7946 §3.1.6).
    Polygon,
    /// Collection of polygon geometries (RFC 7946 §3.1.7).
    MultiPolygon,
    /// Sequence of positions (RFC 7946 §3.1.4).
    LineString,
}

impl GeoJsonType {
    /// Return the RFC 7946 canonical type string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Point => "Point",
            Self::Polygon => "Polygon",
            Self::MultiPolygon => "MultiPolygon",
            Self::LineString => "LineString",
        }
    }

    /// Map a PostGIS geometry type string to the corresponding variant.
    ///
    /// Unknown strings default to [`GeoJsonType::Point`] as a defensive
    /// fallback; callers that require an exact match should validate
    /// `as_str()` after construction.
    pub fn from_db_str(s: &str) -> Self {
        match s {
            "Polygon" => Self::Polygon,
            "MultiPolygon" => Self::MultiPolygon,
            "LineString" => Self::LineString,
            _ => Self::Point,
        }
    }
}

/// GeoJSON geometry (flexible via `serde_json::Value` for coordinates).
///
/// Using `serde_json::Value` for `coordinates` avoids a family of geometry
/// wrapper types (Point, Polygon, MultiPolygon …) without sacrificing
/// correctness — MapLibre GL accepts the raw JSON unchanged.
#[derive(Debug, Clone)]
pub struct GeoJsonGeometry {
    /// GeoJSON geometry type; encodes the RFC 7946 §3.1 type discriminator.
    pub r#type: GeoJsonType,
    /// Raw coordinate array; shape depends on `type`.
    pub coordinates: serde_json::Value,
}

/// Raw land price observation used as input for TLS scoring and trend analysis.
///
/// Sourced from the `land_prices` PostGIS table. Fields are raw SQL types
/// rather than validated newtypes because this struct is an internal
/// intermediate value, never exposed at API boundaries.
#[derive(Debug, Clone)]
pub struct PriceRecord {
    /// Survey year for this price observation.
    pub year: i32,
    /// Land price in JPY per square metre.
    pub price_per_sqm: i64,
}

/// Single data point in a price trend time series.
#[derive(Debug, Clone)]
pub struct TrendPoint {
    /// Survey year.
    pub year: Year,
    /// Land price in JPY per square metre for that year.
    pub price_per_sqm: PricePerSqm,
}

/// Nearest observation point metadata attached to a trend response.
#[derive(Debug, Clone)]
pub struct TrendLocation {
    /// Human-readable address of the nearest land price point.
    pub address: Address,
    /// Distance in metres from the queried coordinate to the observation point.
    pub distance_m: f64,
}

/// Re-exported from `terrasight-domain` so downstream crates use a single
/// canonical type for land price aggregate statistics.
pub use terrasight_domain::types::{LandPriceStats, RiskStats};

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

/// Health check result for the `/api/v1/health` endpoint.
///
/// Moved to the domain layer (P0 polish) so the handler does not need to
/// import from the usecase layer.
#[derive(Debug, Clone)]
pub struct HealthStatus {
    /// Overall status string: `"ok"` or `"degraded"`.
    ///
    /// See `constants::HEALTH_STATUS_OK` and `constants::HEALTH_STATUS_DEGRADED`.
    pub status: &'static str,
    /// `true` when the PostgreSQL `SELECT 1` probe succeeded.
    pub db_connected: bool,
    /// `true` when the `REINFOLIB_API_KEY` environment variable is set.
    pub reinfolib_key_set: bool,
    /// Crate version string from `CARGO_PKG_VERSION`.
    pub version: &'static str,
}

/// School accessibility details used in TLS S2 (education sub-score).
///
/// Collected within an 800 m radius of the scored coordinate by
/// [`TlsRepository::find_schools_nearby`](crate::domain::repository::TlsRepository::find_schools_nearby).
#[derive(Debug, Clone)]
pub struct SchoolStats {
    /// Total number of school facilities within 800 m.
    pub count_800m: i64,
    /// `true` when at least one elementary school (小学校) is present.
    pub has_primary: bool,
    /// `true` when at least one junior-high school (中学校) is present.
    pub has_junior_high: bool,
}

/// Medical facility details used in TLS S3 (medical sub-score).
///
/// Collected within a 1 000 m radius of the scored coordinate by
/// [`TlsRepository::find_medical_nearby`](crate::domain::repository::TlsRepository::find_medical_nearby).
#[derive(Debug, Clone)]
pub struct MedicalStats {
    /// Number of hospitals (病院, ≥ 20 beds) within 1 000 m.
    pub hospital_count: i64,
    /// Number of clinics (診療所, < 20 beds) within 1 000 m.
    pub clinic_count: i64,
    /// Sum of licensed bed counts across all hospitals within 1 000 m.
    pub total_beds: i64,
}

/// Z-score of a land price observation relative to all prices in the same
/// zoning type.
///
/// Used in TLS P2 (price attractiveness) to flag statistically cheap
/// locations within their zoning class.
#[derive(Debug, Clone)]
pub struct ZScoreResult {
    /// Standardised score. Negative values are below the zoning-type mean.
    pub z_score: f64,
    /// JIS zoning type code used as the comparison population.
    pub zone_type: String,
    /// Number of records in the comparison population.
    pub sample_count: i64,
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

/// Raw land-price record sourced from the repository layer before TLS
/// enrichment. Mirrors the columns selected by
/// [`LandPriceRepository::find_for_opportunities`](crate::domain::repository::LandPriceRepository::find_for_opportunities).
#[derive(Debug, Clone)]
pub struct OpportunityRecord {
    /// Database primary key for this land price observation.
    pub id: i64,
    /// Geographic coordinate of the observation point.
    pub coord: Coord,
    /// Street address of the land price survey point.
    pub address: Address,
    /// Urban-planning zone code at this location.
    pub zone: ZoneCode,
    /// Building coverage ratio (建蔽率) at this location.
    pub building_coverage_ratio: BuildingCoverageRatio,
    /// Floor area ratio (容積率) at this location.
    pub floor_area_ratio: FloorAreaRatio,
    /// Land price in JPY per square metre.
    pub price_per_sqm: PricePerSqm,
    /// Survey year for this price record.
    pub year: Year,
}

/// Nearest-station metadata attached to an [`Opportunity`] when available.
///
/// `None` when the land price point has no nearby rail station in the database.
#[derive(Debug, Clone)]
pub struct StationHint {
    /// Station name (e.g. `"新宿駅"`).
    pub name: AreaName,
    /// Walking distance from the observation point to the station entrance.
    pub distance: Meters,
}

/// TLS-enriched investment opportunity returned by `GetOpportunitiesUsecase`.
///
/// Composed from an [`OpportunityRecord`] plus the scoring pipeline output
/// ([`TlsScore`], [`RiskLevel`], [`OpportunitySignal`]) and an optional
/// 5-year price-change percentage.
#[derive(Debug, Clone)]
pub struct Opportunity {
    /// Raw database record before enrichment.
    pub record: OpportunityRecord,
    /// Composite Total Location Score (0–100).
    pub tls: TlsScore,
    /// Disaster risk bucket derived from the S1 sub-score.
    pub risk: RiskLevel,
    /// Investment signal bucket derived from TLS and risk together.
    pub signal: OpportunitySignal,
    /// 5-year CAGR as a percentage (negative = price decline).
    pub trend_pct: Percent,
    /// Nearest rail station, if available.
    pub station: Option<StationHint>,
}

/// Cached result of opportunity TLS enrichment and filtering.
///
/// The usecase caches the full filtered pool keyed on
/// [`OpportunitiesCacheKey`](crate::domain::value_object::OpportunitiesCacheKey);
/// the handler applies `limit`/`offset` pagination after cache retrieval so
/// all pagination pages share a single expensive cache entry.
#[derive(Debug, Clone, Default)]
pub struct CachedOpportunitiesResponse {
    /// TLS-enriched, filtered, and sorted opportunities.
    pub items: Vec<Opportunity>,
    /// Total count before pagination (for `X-Total-Count` header).
    pub total: usize,
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
