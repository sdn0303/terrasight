//! Handler-layer response DTOs, grouped by endpoint.
//!
//! Each submodule owns the `#[derive(Serialize)]` types returned by a
//! specific endpoint. By convention, top-level response structs carry a
//! `Response` or `Dto` suffix (e.g. [`AreaDataResponseDto`],
//! [`StatsResponse`]) to distinguish them from domain entities.
//!
//! ## Naming convention
//!
//! | Suffix | Usage |
//! |--------|-------|
//! | `ResponseDto` | Multi-field envelope or transparent wrapper |
//! | `Response` | Direct mapping of a domain aggregate |
//! | `Dto` | Nested sub-object inside a response |
//!
//! ## GeoJSON responses
//!
//! [`LayerResponseDto`] is a GeoJSON `FeatureCollection` augmented with
//! `truncated`, `count`, and `limit` metadata fields so that MapLibre GL
//! clients can detect when the server has capped the result set and prompt
//! the user to zoom in.

pub(crate) mod appraisal;
pub(crate) mod area_data;
pub(crate) mod area_stats;
pub(crate) mod health;
pub(crate) mod layer;
pub(crate) mod municipality;
pub(crate) mod opportunities;
pub(crate) mod population;
pub(crate) mod stats;
pub(crate) mod tls;
pub(crate) mod transaction;
pub(crate) mod trend;
pub(crate) mod vacancy;

pub(crate) use area_data::AreaDataResponseDto;
pub(crate) use area_stats::AreaStatsResponse;
pub(crate) use health::HealthResponse;
pub(crate) use layer::LayerResponseDto;
pub(crate) use opportunities::OpportunitiesResponseDto;
pub(crate) use stats::StatsResponse;
pub(crate) use tls::TlsResponse;
pub(crate) use trend::TrendResponse;
