//! Handler-layer request DTOs, grouped by endpoint.
//!
//! Each submodule owns the `#[derive(Deserialize)]` type used by axum's
//! `Query` extractor plus an `into_domain` method that converts the raw
//! query string into validated domain value objects. Tests are colocated
//! in each submodule.

pub mod appraisal;
pub mod area_data;
pub mod area_stats;
pub mod bbox;
pub mod land_price;
pub mod municipality;
pub mod opportunities;
pub mod transaction;
pub mod trend;

pub use appraisal::AppraisalsQuery;
pub use area_data::AreaDataQuery;
pub use area_stats::AreaStatsQuery;
pub use bbox::{BBoxQuery, CoordQuery};
pub use land_price::{LandPriceByYearRangeQuery, LandPriceQuery};
pub use municipality::MunicipalitiesQuery;
pub use opportunities::{OpportunitiesFilters, OpportunitiesQuery};
pub use transaction::{TransactionSummaryQuery, TransactionsQuery};
pub use trend::TrendQuery;
