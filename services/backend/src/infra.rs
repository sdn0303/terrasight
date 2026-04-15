//! Infrastructure layer: PostgreSQL + PostGIS repository implementations.
//!
//! Each `pg_*.rs` module implements a domain repository trait from
//! [`crate::domain::repository`] using [`sqlx`] against a [`sqlx::PgPool`].
//! Spatial queries rely on PostGIS functions (`ST_Intersects`, `ST_MakeEnvelope`,
//! `ST_DWithin`, `ST_AsGeoJSON`, …); results are converted to domain types via
//! [`geo_convert`] and [`row_types`].
//!
//! ## Sub-modules
//!
//! | Module | Implements |
//! |--------|-----------|
//! | [`pg_area_repository`] | [`LayerRepository`](crate::domain::repository::LayerRepository) |
//! | [`pg_land_price_repository`] | [`LandPriceRepository`](crate::domain::repository::LandPriceRepository) |
//! | [`pg_stats_repository`] | [`StatsRepository`](crate::domain::repository::StatsRepository) |
//! | [`pg_tls_repository`] | [`TlsRepository`](crate::domain::repository::TlsRepository) |
//! | [`pg_trend_repository`] | [`TrendRepository`](crate::domain::repository::TrendRepository) |
//! | [`pg_admin_area_stats_repository`] | [`AdminAreaStatsRepository`](crate::domain::repository::AdminAreaStatsRepository) |
//! | [`pg_aggregation_repository`] | [`AggregationRepository`](crate::domain::repository::AggregationRepository) |
//! | [`pg_health_repository`] | [`HealthRepository`](crate::domain::repository::HealthRepository) |
//! | [`pg_appraisal_repository`] | [`AppraisalRepository`](crate::domain::repository::AppraisalRepository) |
//! | [`pg_municipality_repository`] | [`MunicipalityRepository`](crate::domain::repository::MunicipalityRepository) |
//! | [`pg_transaction_repository`] | [`TransactionRepository`](crate::domain::repository::TransactionRepository) |
//! | [`query_helpers`] | Shared `run_query` timeout wrapper and `apply_limit` N+1 helper |
//! | [`row_types`] | Shared `FromRow` structs used across repositories |
//! | [`geo_convert`] | PostGIS `ST_AsGeoJSON` → domain [`GeoFeature`](crate::domain::model::GeoFeature) |
//! | [`map_db_err`] | `sqlx::Error` → [`DomainError::Database`](crate::domain::error::DomainError) |
//! | [`opportunities_cache`] | In-memory TTL cache for `/api/v1/opportunities` |
//! | [`reinfolib_mock`] | [`ReinfolibDataSource`](crate::domain::reinfolib::ReinfolibDataSource) implementations and factory |

pub(crate) mod geo_convert;
pub(crate) mod map_db_err;
pub(crate) mod opportunities_cache;
pub(crate) mod pg_admin_area_stats_repository;
pub(crate) mod pg_aggregation_repository;
pub(crate) mod pg_appraisal_repository;
pub(crate) mod pg_area_repository;
pub(crate) mod pg_health_repository;
pub(crate) mod pg_land_price_repository;
pub(crate) mod pg_municipality_repository;
pub(crate) mod pg_stats_repository;
pub(crate) mod pg_tls_repository;
pub(crate) mod pg_transaction_repository;
pub(crate) mod pg_trend_repository;
pub(crate) mod query_helpers;
pub(crate) mod reinfolib_mock;
pub(crate) mod row_types;
