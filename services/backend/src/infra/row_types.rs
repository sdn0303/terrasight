//! Shared sqlx `FromRow` structs used across multiple repositories.

use crate::domain::entity::LandPriceStats;

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct CountRow {
    pub(crate) count: i64,
}

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct AreaRow {
    pub(crate) area: f64,
}

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct LandPriceStatsRow {
    pub(crate) avg_price: Option<f64>,
    pub(crate) median_price: Option<f64>,
    pub(crate) min_price: Option<i64>,
    pub(crate) max_price: Option<i64>,
    pub(crate) count: i64,
}

impl From<LandPriceStatsRow> for LandPriceStats {
    fn from(row: LandPriceStatsRow) -> Self {
        Self {
            avg_per_sqm: row.avg_price,
            median_per_sqm: row.median_price,
            min_per_sqm: row.min_price,
            max_per_sqm: row.max_price,
            count: row.count,
        }
    }
}
