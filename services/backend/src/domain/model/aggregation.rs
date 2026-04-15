//! Polygon-level aggregation rows for choropleth map layers.
//!
//! These are domain value objects produced by the aggregation repository
//! and consumed by the usecase layer to assemble GeoJSON FeatureCollections.
//! Derived metrics (e.g. `change_pct`) are computed in the usecase layer,
//! not in the database query.

/// Land price statistics aggregated per municipality polygon.
///
/// Returned by [`AggregationRepository::land_price_aggregation`] as one row
/// per `admin_boundaries` municipality that intersects the query bounding box.
///
/// The `geometry` field carries the raw `ST_AsGeoJSON` output for the
/// polygon; the usecase layer converts it to a [`GeoFeature`](super::GeoFeature).
#[derive(Debug, Clone)]
pub struct LandPriceAggRow {
    /// 5-digit JIS administrative code (e.g. `"13101"`).
    pub admin_code: String,
    /// Prefecture display name (e.g. `"東京都"`).
    pub pref_name: String,
    /// Municipality display name (e.g. `"千代田区"`).
    pub city_name: String,
    /// Raw GeoJSON geometry from `ST_AsGeoJSON(ab.geom)::jsonb`.
    pub geometry: serde_json::Value,
    /// Mean land price per m² for the latest survey year.
    pub avg_price: f64,
    /// Median (50th percentile) land price per m².
    pub median_price: f64,
    /// Minimum land price per m².
    pub min_price: f64,
    /// Maximum land price per m².
    pub max_price: f64,
    /// Number of land price points within the polygon.
    pub count: i32,
    /// Mean land price per m² for the previous survey year.
    pub prev_year_avg: f64,
}

impl LandPriceAggRow {
    /// Compute year-over-year change percentage.
    ///
    /// Returns `0.0` when `prev_year_avg` is zero (division guard).
    /// Rounded to 1 decimal place.
    pub fn change_pct(&self) -> f64 {
        match self.prev_year_avg {
            prev if prev > 0.0 => ((self.avg_price - prev) / prev * 100.0 * 10.0).round() / 10.0,
            _ => 0.0,
        }
    }
}

/// Transaction statistics aggregated per municipality polygon.
///
/// Returned by [`AggregationRepository::transaction_aggregation`] as one row
/// per `admin_boundaries` municipality that intersects the query bounding box.
#[derive(Debug, Clone)]
pub struct TransactionAggRow {
    /// 5-digit JIS administrative code.
    pub admin_code: String,
    /// Municipality display name.
    pub city_name: String,
    /// Raw GeoJSON geometry from `ST_AsGeoJSON(ab.geom)::jsonb`.
    pub geometry: serde_json::Value,
    /// Number of transactions within the polygon.
    pub tx_count: i32,
    /// Mean transaction price per m² (JPY).
    pub avg_price_sqm: f64,
    /// Mean total transaction price (JPY).
    pub avg_total_price: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn change_pct_normal() {
        let row = LandPriceAggRow {
            admin_code: "13101".into(),
            pref_name: "東京都".into(),
            city_name: "千代田区".into(),
            geometry: serde_json::Value::Null,
            avg_price: 1100.0,
            median_price: 1000.0,
            min_price: 500.0,
            max_price: 2000.0,
            count: 10,
            prev_year_avg: 1000.0,
        };
        assert!((row.change_pct() - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn change_pct_zero_prev() {
        let row = LandPriceAggRow {
            admin_code: "13101".into(),
            pref_name: "東京都".into(),
            city_name: "千代田区".into(),
            geometry: serde_json::Value::Null,
            avg_price: 1100.0,
            median_price: 1000.0,
            min_price: 500.0,
            max_price: 2000.0,
            count: 10,
            prev_year_avg: 0.0,
        };
        assert!((row.change_pct()).abs() < f64::EPSILON);
    }
}
