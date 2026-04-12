//! Response DTOs for `GET /api/v1/appraisals`.

use serde::Serialize;

use crate::domain::appraisal::AppraisalDetail;

/// Single appraisal record returned in the response array.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppraisalDetailResponse {
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

impl From<AppraisalDetail> for AppraisalDetailResponse {
    fn from(d: AppraisalDetail) -> Self {
        Self {
            city_code: d.city_code,
            city_name: d.city_name,
            address: d.address,
            land_use_code: d.land_use_code,
            price_per_sqm: d.price_per_sqm,
            appraisal_price: d.appraisal_price,
            lot_area_sqm: d.lot_area_sqm,
            zone_code: d.zone_code,
            building_coverage: d.building_coverage,
            floor_area_ratio: d.floor_area_ratio,
            comparable_price: d.comparable_price,
            yield_price: d.yield_price,
            cost_price: d.cost_price,
            fudosan_id: d.fudosan_id,
        }
    }
}
