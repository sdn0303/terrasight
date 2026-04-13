/// 取引価格集計（マテリアライズドビュー由来）。
#[derive(Debug, Clone)]
pub struct TransactionSummary {
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

/// 取引価格明細。
#[derive(Debug, Clone)]
pub struct TransactionDetail {
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
