//! Land price domain types: raw observation records and aggregate statistics.

/// Raw land price observation used as input for TLS scoring and trend analysis.
///
/// Sourced from the `land_prices` PostGIS table. Fields are raw SQL types
/// rather than validated newtypes because this struct is an internal
/// intermediate value, never exposed at API boundaries.
#[derive(Debug, Clone)]
pub struct PriceRecord {
    /// Survey year for this price observation.
    pub year: i32,
    /// Land price in JPY per square metre.
    pub price_per_sqm: i64,
}

/// Re-exported from `terrasight-domain` so downstream crates use a single
/// canonical type for land price aggregate statistics.
pub use terrasight_domain::types::LandPriceStats;
