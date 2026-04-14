//! Investment opportunity domain types: raw records, enriched results, signals,
//! filters, pagination, and cache keys for the `/api/v1/opportunities` endpoint.

use terrasight_domain::scoring::tls::WeightPreset;

use super::primitives::{
    Address, AreaName, BBox, BuildingCoverageRatio, CityCode, Coord, FloorAreaRatio, Meters,
    Percent, PrefCode, PricePerSqm, Year, ZoneCode,
};
use super::tls::{RiskLevel, TlsScore};

/// Opportunity signal bucket: `Hot | Warm | Neutral | Cold`.
///
/// Derived from the combination of TLS score and risk level. See
/// [`OpportunitySignal::derive`] for the exact mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpportunitySignal {
    /// Low risk + TLS ≥ 80. Strong buy signal for investment analysis.
    Hot,
    /// Low or mid risk + TLS ≥ 65. Good potential with moderate caution.
    Warm,
    /// TLS ≥ 50 (any risk). Average location; further due diligence required.
    Neutral,
    /// TLS < 50. Below-average fundamentals; not recommended for investment.
    Cold,
}

impl OpportunitySignal {
    /// Return the canonical REST API string for this signal.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Hot => "hot",
            Self::Warm => "warm",
            Self::Neutral => "neutral",
            Self::Cold => "cold",
        }
    }

    /// Derive a signal from a TLS score and risk level.
    ///
    /// High-risk locations are never classified as hotter than `Neutral`,
    /// regardless of TLS score.
    pub fn derive(tls: TlsScore, risk: RiskLevel) -> Self {
        use terrasight_domain::scoring::constants::{
            SIGNAL_HOT_MIN_TLS, SIGNAL_NEUTRAL_MIN_TLS, SIGNAL_WARM_MIN_TLS,
        };
        let score = tls.value();
        match (score, risk) {
            (s, RiskLevel::Low) if s >= SIGNAL_HOT_MIN_TLS => Self::Hot,
            (s, RiskLevel::Low | RiskLevel::Mid) if s >= SIGNAL_WARM_MIN_TLS => Self::Warm,
            (s, _) if s >= SIGNAL_NEUTRAL_MIN_TLS => Self::Neutral,
            _ => Self::Cold,
        }
    }
}

/// Clamped page-size parameter for the opportunities endpoint.
///
/// Enforces `1 <= value <= MAX_OPPORTUNITY_LIMIT` by clamping; construction is
/// infallible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpportunityLimit(u32);

impl OpportunityLimit {
    /// Server-enforced maximum page size.
    pub const MAX: u32 = crate::domain::constants::MAX_OPPORTUNITY_LIMIT;
    /// Default page size used when the client omits the `limit` parameter.
    pub const DEFAULT: Self = Self(crate::domain::constants::DEFAULT_OPPORTUNITY_LIMIT);

    /// Clamp a raw page-size value to `[1, MAX]`.
    pub fn clamped(value: u32) -> Self {
        Self(value.clamp(1, Self::MAX))
    }

    /// Return the page size as a `u32`.
    pub fn get(self) -> u32 {
        self.0
    }
}

/// Offset parameter for paginated opportunities responses.
///
/// No upper bound — callers paginate until the response is shorter than
/// `OpportunityLimit::get()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpportunityOffset(u32);

impl OpportunityOffset {
    /// Wrap a zero-based page offset.
    ///
    /// No upper bound is enforced; the usecase returns an empty slice when
    /// `offset` exceeds the size of the cached opportunity pool.
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    /// Return the offset as a `u32`.
    pub fn get(self) -> u32 {
        self.0
    }
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
/// [`OpportunitiesCacheKey`]; the handler applies `limit`/`offset` pagination
/// after cache retrieval so all pagination pages share a single expensive
/// cache entry.
#[derive(Debug, Clone, Default)]
pub struct CachedOpportunitiesResponse {
    /// TLS-enriched, filtered, and sorted opportunities.
    pub items: Vec<Opportunity>,
    /// Total count before pagination (for `X-Total-Count` header).
    pub total: usize,
}

/// Validated filter set for opportunity queries.
///
/// Constructed by the handler layer from raw query parameters and passed to
/// the usecase. All fields have been validated and normalised; the usecase
/// can use them directly without further checking.
#[derive(Debug, Clone)]
pub struct OpportunitiesFilters {
    /// Geographic bounding box for the query.
    pub bbox: BBox,
    /// Maximum number of opportunities to return (after cache + pagination).
    pub limit: OpportunityLimit,
    /// Zero-based page offset into the cached opportunity pool.
    pub offset: OpportunityOffset,
    /// Minimum acceptable TLS score. `None` means no lower bound.
    pub tls_min: Option<TlsScore>,
    /// Maximum acceptable risk level. `None` means no upper bound.
    pub risk_max: Option<RiskLevel>,
    /// Allow-list of zone codes. Empty means all zones are accepted.
    pub zones: Vec<ZoneCode>,
    /// Maximum walking distance (m) to the nearest station. `None` means no limit.
    pub station_max: Option<Meters>,
    /// Inclusive price range `(min, max)` in JPY/m². `None` means no price filter.
    pub price_range: Option<(PricePerSqm, PricePerSqm)>,
    /// TLS weight preset controlling sub-score importance.
    pub preset: WeightPreset,
    /// Optional prefecture filter for multi-prefecture deployments.
    pub pref_code: Option<PrefCode>,
    /// Allow-list of 5-digit city codes. Empty means all cities are accepted.
    pub cities: Vec<CityCode>,
}

/// Cache key fingerprint for opportunities requests.
///
/// Excludes `limit` and `offset` so all paginated views of the same filter
/// set share a single in-memory cache slot. Coordinates are stored as integer
/// micro-degrees (`f64 * 1e6` truncated to `i64`) to avoid floating-point
/// equality issues in the `Hash` and `Eq` implementations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OpportunitiesCacheKey {
    /// Bounding box encoded as `(south, west, north, east)` in micro-degrees.
    pub bbox_microdeg: (i64, i64, i64, i64),
    /// Minimum TLS score filter, or `None` if not applied.
    pub tls_min: Option<u8>,
    /// Maximum risk level filter, or `None` if not applied.
    pub risk_max: Option<RiskLevel>,
    /// Zone code allow-list (raw strings for hash stability).
    pub zones: Vec<String>,
    /// Maximum station distance in metres, or `None` if not applied.
    pub station_max: Option<u32>,
    /// Price range as `(min_jpy, max_jpy)` per m², or `None` if not applied.
    pub price_range: Option<(i64, i64)>,
    /// TLS weight preset identifier.
    pub preset: WeightPreset,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opportunity_limit_clamps() {
        assert_eq!(OpportunityLimit::clamped(0).get(), 1);
        assert_eq!(OpportunityLimit::clamped(1).get(), 1);
        assert_eq!(OpportunityLimit::clamped(50).get(), 50);
        assert_eq!(OpportunityLimit::clamped(200).get(), 50);
        assert_eq!(OpportunityLimit::DEFAULT.get(), 50);
    }

    #[test]
    fn opportunity_offset_preserved() {
        assert_eq!(OpportunityOffset::new(0).get(), 0);
        assert_eq!(OpportunityOffset::new(100).get(), 100);
    }

    #[test]
    fn opportunity_signal_derive_table() {
        let tls = |v: u8| TlsScore::from_f64_clamped(v as f64);
        assert_eq!(
            OpportunitySignal::derive(tls(85), RiskLevel::Low),
            OpportunitySignal::Hot
        );
        assert_eq!(
            OpportunitySignal::derive(tls(90), RiskLevel::High),
            OpportunitySignal::Neutral
        );
        assert_eq!(
            OpportunitySignal::derive(tls(70), RiskLevel::Low),
            OpportunitySignal::Warm
        );
        assert_eq!(
            OpportunitySignal::derive(tls(65), RiskLevel::Mid),
            OpportunitySignal::Warm
        );
        assert_eq!(
            OpportunitySignal::derive(tls(55), RiskLevel::High),
            OpportunitySignal::Neutral
        );
        assert_eq!(
            OpportunitySignal::derive(tls(40), RiskLevel::Low),
            OpportunitySignal::Cold
        );
    }
}
