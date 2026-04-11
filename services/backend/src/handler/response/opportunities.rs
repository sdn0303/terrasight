//! Response DTOs for `GET /api/v1/opportunities`.
//!
//! The handler never serializes a domain [`Opportunity`] directly —
//! instead, it maps through [`OpportunityDto`] which converts every
//! newtype back into a JSON-friendly scalar (e.g. [`TlsScore`] -> `u8`,
//! [`RiskLevel`] -> `&'static str`).

use std::sync::Arc;

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

impl From<&CachedOpportunitiesResponse> for OpportunitiesResponseDto {
    fn from(cached: &CachedOpportunitiesResponse) -> Self {
        Self {
            items: cached.items.iter().map(OpportunityDto::from).collect(),
            total: cached.total,
            truncated: cached.truncated,
        }
    }
}

impl From<Arc<CachedOpportunitiesResponse>> for OpportunitiesResponseDto {
    fn from(cached: Arc<CachedOpportunitiesResponse>) -> Self {
        Self::from(&*cached)
    }
}
