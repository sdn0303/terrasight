//! Response DTOs for the transaction endpoints.
//!
//! `TransactionSummaryResponse` is returned by `GET /api/v1/transactions/summary`.
//! `TransactionDetailResponse` is returned by `GET /api/v1/transactions`.

use serde::Serialize;

use crate::domain::transaction::{TransactionDetail, TransactionSummary};

/// Response item for `GET /api/v1/transactions/summary`.
///
/// Each item represents one `(city_code, transaction_year, property_type)` bucket.
#[derive(Debug, Serialize)]
pub struct TransactionSummaryResponse {
    /// 5-digit municipality code (e.g. `"13101"`).
    pub city_code: String,
    /// Calendar year of the transactions in this bucket.
    pub transaction_year: i16,
    /// MLIT property type string in Japanese (e.g. `"宅地(土地)"`).
    pub property_type: String,
    /// Number of transactions in this bucket.
    pub tx_count: i32,
    /// Mean total transaction price in JPY for this bucket.
    pub avg_total_price: i64,
    /// Median total transaction price in JPY. Equals `avg_total_price` when
    /// the bucket contains a single record.
    pub median_total_price: i64,
    /// Mean price per square metre in JPY. `null` when area data is unavailable.
    pub avg_price_sqm: Option<i32>,
    /// Mean lot/floor area in square metres. `null` when area data is unavailable.
    pub avg_area_sqm: Option<i32>,
    /// Mean walk time from the nearest station in minutes. `null` when not reported.
    pub avg_walk_min: Option<i16>,
}

impl From<TransactionSummary> for TransactionSummaryResponse {
    fn from(s: TransactionSummary) -> Self {
        Self {
            city_code: s.city_code,
            transaction_year: s.transaction_year,
            property_type: s.property_type,
            tx_count: s.tx_count,
            avg_total_price: s.avg_total_price,
            median_total_price: s.median_total_price,
            avg_price_sqm: s.avg_price_sqm,
            avg_area_sqm: s.avg_area_sqm,
            avg_walk_min: s.avg_walk_min,
        }
    }
}

/// Response item for `GET /api/v1/transactions`.
///
/// Each item represents one individual real estate transaction record.
#[derive(Debug, Serialize)]
pub struct TransactionDetailResponse {
    /// 5-digit municipality code of the property location.
    pub city_code: String,
    /// Municipality name in Japanese.
    pub city_name: String,
    /// District / chome name within the municipality. `null` when not reported.
    pub district_name: Option<String>,
    /// MLIT property type string in Japanese (e.g. `"宅地(土地)"`).
    pub property_type: String,
    /// Total transaction price in JPY (ten-thousand yen units in source data,
    /// converted to full JPY here).
    pub total_price: i64,
    /// Price per square metre in JPY. `null` when area data is unavailable.
    pub price_per_sqm: Option<i32>,
    /// Land or floor area in square metres. `null` when not reported.
    pub area_sqm: Option<i32>,
    /// Floor plan code (e.g. `"3LDK"`). `null` for land-only transactions.
    pub floor_plan: Option<String>,
    /// Year the building was constructed. `null` for land-only transactions.
    pub building_year: Option<i16>,
    /// Building structure type in Japanese (e.g. `"RC"`). `null` when not reported.
    pub building_structure: Option<String>,
    /// Name of the nearest railway station. `null` when not reported.
    pub nearest_station: Option<String>,
    /// Walk time from the nearest station in minutes. `null` when not reported.
    pub station_walk_min: Option<i16>,
    /// Transaction quarter in ISO-like format (e.g. `"2023Q1"`).
    pub transaction_quarter: String,
}

impl From<TransactionDetail> for TransactionDetailResponse {
    fn from(d: TransactionDetail) -> Self {
        Self {
            city_code: d.city_code,
            city_name: d.city_name,
            district_name: d.district_name,
            property_type: d.property_type,
            total_price: d.total_price,
            price_per_sqm: d.price_per_sqm,
            area_sqm: d.area_sqm,
            floor_plan: d.floor_plan,
            building_year: d.building_year,
            building_structure: d.building_structure,
            nearest_station: d.nearest_station,
            station_walk_min: d.station_walk_min,
            transaction_quarter: d.transaction_quarter,
        }
    }
}
