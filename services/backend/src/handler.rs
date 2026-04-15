//! HTTP handler layer for the Terrasight API.
//!
//! Each submodule corresponds to one API endpoint group. Handlers are thin
//! adapters: they extract and validate the incoming request via Axum's
//! [`Query`](axum::extract::Query) / [`State`](axum::extract::State)
//! extractors, delegate all business logic to a usecase, and map
//! [`DomainError`](crate::domain::error::DomainError) to an HTTP status
//! code via [`AppError`](crate::handler::error::AppError).
//!
//! ## Endpoint index
//!
//! | Module | Route |
//! |--------|-------|
//! | [`appraisals`] | `GET /api/v1/appraisals` |
//! | [`area_data`] | `GET /api/v1/area-data` |
//! | [`area_stats`] | `GET /api/v1/area-stats` |
//! | [`health`] | `GET /api/v1/health` |
//! | [`land_price`] | `GET /api/v1/land-prices` |
//! | [`land_price_aggregation`] | `GET /api/v1/land-prices/aggregation` |
//! | [`land_price_by_year_range`] | `GET /api/v1/land-prices/all-years` |
//! | [`municipalities`] | `GET /api/v1/municipalities` |
//! | [`opportunities`] | `GET /api/v1/opportunities` |
//! | [`score`] | `GET /api/v1/score` |
//! | [`stats`] | `GET /api/v1/stats` |
//! | [`transaction_aggregation`] | `GET /api/v1/transactions/aggregation` |
//! | [`transaction_summary`] | `GET /api/v1/transactions/summary` |
//! | [`transactions`] | `GET /api/v1/transactions` |
//! | [`trend`] | `GET /api/v1/trend` |
//!
//! ## Supporting modules
//!
//! - [`error`] — [`AppError`](error::AppError) type alias and
//!   [`ErrorMapping`](terrasight_server::http::error::ErrorMapping) impl
//! - [`request`] — Axum `Query` extractor DTOs and `into_domain` / `into_filters` conversions
//! - [`response`] — `Serialize` response DTOs

pub(crate) mod appraisals;
pub(crate) mod area_data;
pub(crate) mod area_stats;
pub(crate) mod error;
pub(crate) mod health;
pub(crate) mod land_price;
pub(crate) mod land_price_aggregation;
pub(crate) mod land_price_by_year_range;
pub(crate) mod municipalities;
pub(crate) mod opportunities;
pub(crate) mod request;
pub(crate) mod response;
pub(crate) mod score;
pub(crate) mod stats;
pub(crate) mod transaction_aggregation;
pub(crate) mod transaction_summary;
pub(crate) mod transactions;
pub(crate) mod trend;
