//! Handler-layer response DTOs, grouped by endpoint.
//!
//! Each submodule owns the `#[derive(Serialize)]` types returned by a
//! specific endpoint.

pub(crate) mod appraisal;
pub(crate) mod area_data;
pub(crate) mod area_stats;
pub(crate) mod health;
pub(crate) mod layer;
pub(crate) mod municipality;
pub(crate) mod opportunities;
pub(crate) mod stats;
pub(crate) mod tls;
pub(crate) mod transaction;
pub(crate) mod trend;

pub(crate) use area_data::AreaDataResponseDto;
pub(crate) use area_stats::AreaStatsResponse;
pub(crate) use health::HealthResponse;
pub(crate) use layer::LayerResponseDto;
pub(crate) use opportunities::OpportunitiesResponseDto;
pub(crate) use stats::StatsResponse;
pub(crate) use tls::TlsResponse;
pub(crate) use trend::TrendResponse;
