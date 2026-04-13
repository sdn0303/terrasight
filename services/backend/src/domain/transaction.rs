//! Real-estate transaction domain types.
//!
//! Transaction data originates from the MLIT reinfolib API (not 不動産情報
//! ライブラリ land price surveys) and covers actual sale/purchase prices for
//! residential and commercial properties. Two granularities are exposed:
//!
//! - [`TransactionSummary`] — city-level aggregate from a materialised view.
//! - [`TransactionDetail`] — individual transaction records.

/// Aggregated real-estate transaction statistics per city, year, and property type.
///
/// Sourced from the `transaction_summaries` materialised view, which is
/// refreshed periodically by the data pipeline. Used by the
/// `/api/v1/transactions/summary` endpoint.
#[derive(Debug, Clone)]
pub struct TransactionSummary {
    /// JIS X 0402 5-digit city code for this aggregate group.
    pub city_code: String,
    /// Calendar year of the transactions in this group.
    pub transaction_year: i16,
    /// MLIT property type string (e.g. `"宅地(土地)"`, `"中古マンション等"`).
    pub property_type: String,
    /// Number of transactions in this group.
    pub tx_count: i32,
    /// Average total transaction price in JPY for this group.
    pub avg_total_price: i64,
    /// Median total transaction price in JPY for this group.
    pub median_total_price: i64,
    /// Average price per square metre in JPY, if available.
    pub avg_price_sqm: Option<i32>,
    /// Average building area in square metres, if available.
    pub avg_area_sqm: Option<i32>,
    /// Average walking time (minutes) to the nearest station, if available.
    pub avg_walk_min: Option<i16>,
}

/// Individual real-estate transaction record.
///
/// Sourced from the `transactions` table. Used by the
/// `/api/v1/transactions` endpoint for city-level drilldown.
#[derive(Debug, Clone)]
pub struct TransactionDetail {
    /// JIS X 0402 5-digit city code for the property's municipality.
    pub city_code: String,
    /// Human-readable city name.
    pub city_name: String,
    /// District or neighbourhood name within the city, if recorded.
    pub district_name: Option<String>,
    /// MLIT property type string.
    pub property_type: String,
    /// Total transaction price in JPY.
    pub total_price: i64,
    /// Unit price per square metre in JPY, if computable.
    pub price_per_sqm: Option<i32>,
    /// Building or land area in square metres, if available.
    pub area_sqm: Option<i32>,
    /// Floor plan description (e.g. `"3LDK"`), if available.
    pub floor_plan: Option<String>,
    /// Year the building was constructed, if available.
    pub building_year: Option<i16>,
    /// Building structure type (e.g. `"RC"`), if available.
    pub building_structure: Option<String>,
    /// Name of the nearest railway station, if available.
    pub nearest_station: Option<String>,
    /// Walking time (minutes) to the nearest station, if available.
    pub station_walk_min: Option<i16>,
    /// Transaction quarter string (e.g. `"2023年第１四半期"`).
    pub transaction_quarter: String,
}
