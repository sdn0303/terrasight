//! Official land appraisal (鑑定評価) domain types.
//!
//! Appraisal records are sourced from the MLIT reinfolib API (or a local
//! PostGIS import) and represent professional property valuation data used
//! by the `/api/v1/appraisals` endpoint. They differ from land price survey
//! data (`land_prices` table) in that they include cost and yield
//! decompositions produced by licensed appraisers.

use crate::domain::entity::{Address, AreaName, ZoneCode};
use crate::domain::value_object::CityCode;

/// Official land appraisal detail record (鑑定評価明細).
///
/// Each record corresponds to a single appraised parcel and includes the
/// appraisal price, lot area, zoning parameters, and the three sub-prices
/// (comparable, yield, cost) used by the appraiser.
#[derive(Debug, Clone)]
pub struct AppraisalDetail {
    /// JIS X 0402 5-digit municipality code for the parcel's location.
    pub city_code: CityCode,
    /// Human-readable municipality name.
    pub city_name: AreaName,
    /// Street address of the appraised parcel.
    pub address: Address,
    /// MLIT land-use classification code (地目コード).
    pub land_use_code: String,
    /// Appraised land price in JPY per square metre.
    pub price_per_sqm: i32,
    /// Total appraised value of the lot in JPY.
    pub appraisal_price: i64,
    /// Lot area in square metres, if available from the source data.
    pub lot_area_sqm: Option<f32>,
    /// Urban-planning zone code at the parcel's location, if available.
    pub zone_code: Option<ZoneCode>,
    /// Building coverage ratio (建蔽率 %) at the parcel's location, if available.
    pub building_coverage: Option<i16>,
    /// Floor area ratio (容積率 %) at the parcel's location, if available.
    pub floor_area_ratio: Option<i16>,
    /// Comparable transaction sub-price in JPY/m², if available.
    pub comparable_price: Option<i32>,
    /// Yield capitalisation sub-price in JPY/m², if available.
    pub yield_price: Option<i32>,
    /// Cost approach sub-price in JPY/m², if available.
    pub cost_price: Option<i32>,
    /// Reinfolib (不動産情報ライブラリ) unique identifier for the record, if available.
    pub fudosan_id: Option<String>,
}
