/// 鑑定評価明細。
#[derive(Debug, Clone)]
pub struct AppraisalDetail {
    pub city_code: String,
    pub city_name: String,
    pub address: String,
    pub land_use_code: String,
    pub price_per_sqm: i32,
    pub appraisal_price: i64,
    pub lot_area_sqm: Option<f32>,
    pub zone_code: Option<String>,
    pub building_coverage: Option<i16>,
    pub floor_area_ratio: Option<i16>,
    pub comparable_price: Option<i32>,
    pub yield_price: Option<i32>,
    pub cost_price: Option<i32>,
    pub fudosan_id: Option<String>,
}
