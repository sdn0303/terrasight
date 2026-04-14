//! Response DTOs for `GET /api/v1/appraisals`.

use serde::Serialize;

use crate::domain::appraisal::AppraisalDetail;

/// Single government appraisal (公示地価) record returned in the response array.
#[derive(Debug, Serialize)]
pub struct AppraisalDetailResponse {
    /// 5-digit municipality code (e.g. `"13101"` for Chiyoda-ku, Tokyo).
    pub city_code: String,
    /// Municipality name in Japanese (e.g. `"千代田区"`).
    pub city_name: String,
    /// Survey point address in Japanese.
    pub address: String,
    /// MLIT land use classification code (e.g. `"住宅地"`, `"商業地"`).
    pub land_use_code: String,
    /// Land price per square metre in JPY as of the survey year.
    pub price_per_sqm: i32,
    /// Total appraised value of the site in JPY.
    pub appraisal_price: i64,
    /// Lot area in square metres. `null` when not reported.
    pub lot_area_sqm: Option<f32>,
    /// Zoning type code from the Urban Planning Act. `null` when not reported.
    pub zone_code: Option<String>,
    /// Building coverage ratio as a percentage (e.g. `60`). `null` when not reported.
    pub building_coverage: Option<i16>,
    /// Floor area ratio as a percentage (e.g. `200`). `null` when not reported.
    pub floor_area_ratio: Option<i16>,
    /// Comparable sales price per square metre in JPY. `null` when not reported.
    pub comparable_price: Option<i32>,
    /// Yield-based appraised price per square metre in JPY. `null` when not reported.
    pub yield_price: Option<i32>,
    /// Cost-approach price per square metre in JPY. `null` when not reported.
    pub cost_price: Option<i32>,
    /// MLIT 不動産情報ライブラリ record identifier. `null` when not linked.
    pub fudosan_id: Option<String>,
}

impl From<AppraisalDetail> for AppraisalDetailResponse {
    fn from(d: AppraisalDetail) -> Self {
        Self {
            city_code: d.city_code.as_str().to_string(),
            city_name: d.city_name.as_str().to_string(),
            address: d.address.as_str().to_string(),
            land_use_code: d.land_use_code,
            price_per_sqm: d.price_per_sqm,
            appraisal_price: d.appraisal_price,
            lot_area_sqm: d.lot_area_sqm,
            zone_code: d.zone_code.as_ref().map(|z| z.as_str().to_string()),
            building_coverage: d.building_coverage,
            floor_area_ratio: d.floor_area_ratio,
            comparable_price: d.comparable_price,
            yield_price: d.yield_price,
            cost_price: d.cost_price,
            fudosan_id: d.fudosan_id,
        }
    }
}
