//! Response DTOs for `GET /api/v1/opportunities`.
//!
//! The handler never serializes a domain [`Opportunity`] directly —
//! instead, it maps through [`OpportunityDto`] which converts every
//! newtype back into a JSON-friendly scalar (e.g. [`TlsScore`] -> `u8`,
//! [`RiskLevel`] -> `&'static str`).

use serde::Serialize;

use crate::domain::entity::Opportunity;
use crate::infra::opportunities_cache::CachedOpportunitiesResponse;

/// Nearest-station metadata attached to an [`OpportunityDto`].
#[derive(Debug, Clone, Serialize)]
pub struct OpportunityStationDto {
    pub name: String,
    pub distance_m: u32,
}

/// TLS-enriched land-price record as serialized to JSON.
///
/// Fields mirror the frontend `Opportunity` TypeScript type and are
/// flattened so the client can map directly into a table row.
#[derive(Debug, Clone, Serialize)]
pub struct OpportunityDto {
    pub id: i64,
    pub lat: f64,
    pub lng: f64,
    pub address: String,
    pub zone: String,
    pub building_coverage_ratio: i32,
    pub floor_area_ratio: i32,
    pub tls: u8,
    pub risk_level: &'static str,
    pub trend_pct: f64,
    pub station: Option<OpportunityStationDto>,
    pub price_per_sqm: i64,
    pub signal: &'static str,
}

/// Top-level response for `GET /api/v1/opportunities`.
///
/// `total` is the number of records that survived TLS enrichment +
/// `tls_min`/`risk_max` filtering (i.e. the full cached pool size).
/// `truncated` is `true` iff there are more records beyond the returned
/// page — clients can detect "has more pages" via this flag without
/// doing an extra request.
#[derive(Debug, Clone, Serialize)]
pub struct OpportunitiesResponseDto {
    pub items: Vec<OpportunityDto>,
    pub total: usize,
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
