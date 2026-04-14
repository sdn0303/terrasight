//! Handler-layer request DTOs, grouped by endpoint.
//!
//! Each submodule owns the `#[derive(Deserialize)]` type consumed by
//! Axum's [`Query`](axum::extract::Query) extractor. Raw query-string
//! values (plain `String`, `f64`, `i32`, …) are validated and converted
//! into strongly-typed domain value objects through two patterns:
//!
//! - **`into_domain(self)`** — used when all fields participate in a single
//!   validation step (e.g. [`BBoxQuery::into_domain`] produces a
//!   [`BBox`](crate::domain::model::BBox)).
//! - **`into_filters(self)`** — used when the query carries a composite
//!   filter set that must be assembled atomically before the usecase can
//!   consume it (e.g. [`OpportunitiesQuery::into_filters`] produces
//!   [`OpportunitiesFilters`](crate::domain::model::OpportunitiesFilters)).
//!
//! Both patterns return `Result<…, DomainError>`, which the handler
//! propagates via `?` and converts to [`AppError`](crate::handler::error::AppError).
//! Unit tests for each conversion are colocated in their submodule.

pub mod appraisal;
pub mod area_data;
pub mod area_stats;
pub mod bbox;
pub mod land_price;
pub mod municipality;
pub mod opportunities;
pub mod transaction;
pub mod trend;

pub use area_data::AreaDataQuery;
pub use area_stats::AreaStatsQuery;
pub use bbox::{BBoxQuery, CoordQuery};
pub use land_price::{LandPriceByYearRangeQuery, LandPriceQuery};
pub use opportunities::OpportunitiesQuery;
pub use trend::TrendQuery;
