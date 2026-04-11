//! Handler-layer response DTOs, grouped by endpoint.
//!
//! Each submodule owns the `#[derive(Serialize)]` types returned by a
//! specific endpoint. A thin `pub use` surface keeps the existing
//! `handler::response::{StatsResponse, LayerResponseDto, …}` import
//! paths working unchanged.

pub mod area_data;
pub mod area_stats;
pub mod health;
pub mod layer;
pub mod stats;
pub mod tls;
pub mod trend;

pub use area_stats::{AreaFacilitiesDto, AreaLandPriceDto, AreaRiskDto, AreaStatsResponse};
pub use health::HealthResponse;
pub use layer::{
    FeatureCollectionDto, FeatureDto, LayerResponseDto, geo_feature_to_dto,
    point_feature_to_polygon,
};
pub use stats::{FacilityStatsDto, LandPriceStatsDto, RiskStatsDto, StatsResponse};
pub use tls::{
    AxesDto, AxisDto, CrossAnalysisDto, LocationDto, SubScoreDto, TlsMetadataDto, TlsResponse,
    TlsSummaryDto,
};
pub use trend::{TrendLocationDto, TrendPointDto, TrendResponse};
