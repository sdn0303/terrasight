//! Response DTOs for the transaction endpoints.
//!
//! `TransactionSummaryResponse` is returned by `GET /api/v1/transactions/summary`.
//! `TransactionDetailResponse` is returned by `GET /api/v1/transactions`.

use serde::Serialize;

use crate::domain::transaction::{TransactionDetail, TransactionSummary};

/// Response item for `GET /api/v1/transactions/summary`.
#[derive(Debug, Serialize)]
pub struct TransactionSummaryResponse {
    pub city_code: String,
    pub transaction_year: i16,
    pub property_type: String,
    pub tx_count: i32,
    pub avg_total_price: i64,
    pub median_total_price: i64,
    pub avg_price_sqm: Option<i32>,
    pub avg_area_sqm: Option<i32>,
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
#[derive(Debug, Serialize)]
pub struct TransactionDetailResponse {
    pub city_code: String,
    pub city_name: String,
    pub district_name: Option<String>,
    pub property_type: String,
    pub total_price: i64,
    pub price_per_sqm: Option<i32>,
    pub area_sqm: Option<i32>,
    pub floor_plan: Option<String>,
    pub building_year: Option<i16>,
    pub building_structure: Option<String>,
    pub nearest_station: Option<String>,
    pub station_walk_min: Option<i16>,
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
