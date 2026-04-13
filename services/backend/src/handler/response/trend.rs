//! Response DTOs for `GET /api/trend`.

use serde::Serialize;

use crate::domain::value_object::TrendAnalysis;

/// Top-level response for `GET /api/v1/trend`.
#[derive(Debug, Serialize)]
pub struct TrendResponse {
    /// Nearest land price survey point selected for the trend computation.
    pub location: TrendLocationDto,
    /// Time series of `(year, price_per_sqm)` observations, sorted ascending by year.
    pub data: Vec<TrendPointDto>,
    /// Compound annual growth rate over the lookback window as a decimal
    /// (e.g. `0.023` for 2.3 % annual growth).
    pub cagr: f64,
    /// Trend direction label: `"rising"`, `"stable"`, or `"falling"`.
    pub direction: String,
}

/// Nearest observation point metadata nested inside [`TrendResponse`].
#[derive(Debug, Serialize)]
pub struct TrendLocationDto {
    /// Human-readable address of the survey point.
    pub address: String,
    /// Straight-line distance from the requested coordinate to the survey
    /// point in metres.
    pub distance_m: f64,
}

/// A single year/price observation inside [`TrendResponse::data`].
#[derive(Debug, Serialize)]
pub struct TrendPointDto {
    /// Survey year (e.g. `2023`).
    pub year: i32,
    /// Land price per square metre for this year in JPY.
    pub price_per_sqm: i64,
}

impl From<TrendAnalysis> for TrendResponse {
    fn from(t: TrendAnalysis) -> Self {
        Self {
            location: TrendLocationDto {
                address: t.location.address.as_str().to_owned(),
                distance_m: t.location.distance_m,
            },
            data: t
                .data
                .into_iter()
                .map(|p| TrendPointDto {
                    year: p.year.value(),
                    price_per_sqm: p.price_per_sqm.value(),
                })
                .collect(),
            cagr: t.cagr,
            direction: t.direction.as_str().to_string(),
        }
    }
}
