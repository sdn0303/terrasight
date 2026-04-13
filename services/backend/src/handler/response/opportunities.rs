//! Response DTOs for `GET /api/v1/opportunities`.
//!
//! The handler never serializes a domain [`Opportunity`] directly —
//! instead, it maps through [`OpportunityDto`] which converts every
//! newtype back into a JSON-friendly scalar (e.g. [`TlsScore`] -> `u8`,
//! [`RiskLevel`] -> `&'static str`).

use serde::Serialize;

use crate::domain::entity::{CachedOpportunitiesResponse, Opportunity};

/// Nearest-station metadata attached to an [`OpportunityDto`].
#[derive(Debug, Clone, Serialize)]
pub struct OpportunityStationDto {
    /// Station name in Japanese (e.g. `"新宿駅"`).
    pub name: String,
    /// Straight-line distance from the land price point to the station in metres.
    pub distance_m: u32,
}

/// TLS-enriched land-price record serialised to JSON.
///
/// Fields mirror the frontend `Opportunity` TypeScript type and are
/// intentionally flat so that the client can map each item directly into
/// a data-table row without nesting traversal.
#[derive(Debug, Clone, Serialize)]
pub struct OpportunityDto {
    /// Database row identifier for the underlying land price record.
    pub id: i64,
    /// Latitude of the land price survey point (WGS-84 decimal degrees).
    pub lat: f64,
    /// Longitude of the land price survey point (WGS-84 decimal degrees).
    pub lng: f64,
    /// Human-readable address string for the survey point.
    pub address: String,
    /// Zoning classification code in Japanese (e.g. `"第一種住居地域"`).
    pub zone: String,
    /// Building coverage ratio as a percentage integer (e.g. `60` for 60 %).
    pub building_coverage_ratio: i32,
    /// Floor area ratio as a percentage integer (e.g. `200` for 200 %).
    pub floor_area_ratio: i32,
    /// Total Location Score on a 0–100 scale.
    pub tls: u8,
    /// Disaster risk level: `"low"`, `"mid"`, or `"high"`.
    pub risk_level: &'static str,
    /// Land price CAGR over the lookback window, expressed as a percentage.
    pub trend_pct: f64,
    /// Nearest railway station. `null` when no station data is available.
    pub station: Option<OpportunityStationDto>,
    /// Land price per square metre in JPY.
    pub price_per_sqm: i64,
    /// Investment signal derived from TLS and trend: `"buy"`, `"hold"`, or `"watch"`.
    pub signal: &'static str,
}

/// Top-level response for `GET /api/v1/opportunities`.
///
/// Supports cursor-free offset pagination. `total` reflects the full
/// filtered pool size so the client can render a page count without an
/// extra `count` request. `truncated` is equivalent to
/// `offset + items.len() < total`.
#[derive(Debug, Clone, Serialize)]
pub struct OpportunitiesResponseDto {
    /// Paginated slice of the filtered opportunity pool.
    pub items: Vec<OpportunityDto>,
    /// Total number of records in the filtered pool (before pagination).
    pub total: usize,
    /// `true` when there are more records beyond this page.
    pub truncated: bool,
}

impl From<&Opportunity> for OpportunityDto {
    fn from(op: &Opportunity) -> Self {
        Self {
            id: op.record.id,
            lat: op.record.coord.lat(),
            lng: op.record.coord.lng(),
            address: op.record.address.as_str().to_string(),
            zone: op.record.zone.as_str().to_string(),
            building_coverage_ratio: op.record.building_coverage_ratio.value(),
            floor_area_ratio: op.record.floor_area_ratio.value(),
            tls: op.tls.value(),
            risk_level: op.risk.as_str(),
            trend_pct: op.trend_pct.value(),
            station: op.station.as_ref().map(|s| OpportunityStationDto {
                name: s.name.as_str().to_string(),
                distance_m: s.distance.value(),
            }),
            price_per_sqm: op.record.price_per_sqm.value(),
            signal: op.signal.as_str(),
        }
    }
}

impl OpportunitiesResponseDto {
    /// Build a paginated response from a cached pool.
    ///
    /// Applies `offset` and `limit` as an in-memory slice of the cached
    /// items, converts the slice to [`OpportunityDto`]s, and computes
    /// `truncated` = *there are records beyond the returned page*.
    pub fn paginated(cached: &CachedOpportunitiesResponse, offset: usize, limit: usize) -> Self {
        let total = cached.total;
        let items: Vec<OpportunityDto> = cached
            .items
            .iter()
            .skip(offset)
            .take(limit)
            .map(OpportunityDto::from)
            .collect();
        let truncated = offset.saturating_add(items.len()) < total;
        Self {
            items,
            total,
            truncated,
        }
    }
}
