//! Handler-layer request DTOs, grouped by endpoint.
//!
//! Each submodule owns the `#[derive(Deserialize)]` type used by axum's
//! `Query` extractor plus an `into_domain` method that converts the raw
//! query string into validated domain value objects. Tests are colocated
//! in each submodule.

pub mod area_data;
pub mod area_stats;
pub mod bbox;
pub mod land_price;
pub mod trend;

pub use area_data::AreaDataQuery;
pub use area_stats::AreaStatsQuery;
pub use bbox::{BBoxQuery, CoordQuery};
pub use land_price::{LandPriceAllYearsQuery, LandPriceQuery};
pub use trend::TrendQuery;
