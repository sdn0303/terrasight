//! Response DTOs for `GET /api/trend`.

use serde::Serialize;

use crate::domain::value_object::TrendAnalysis;

/// Response for `GET /api/trend`.
#[derive(Debug, Serialize)]
pub struct TrendResponse {
    pub location: TrendLocationDto,
    pub data: Vec<TrendPointDto>,
    pub cagr: f64,
    pub direction: String,
}

#[derive(Debug, Serialize)]
pub struct TrendLocationDto {
    pub address: String,
    pub distance_m: f64,
}

#[derive(Debug, Serialize)]
pub struct TrendPointDto {
    pub year: i32,
    pub price_per_sqm: i64,
}

impl From<TrendAnalysis> for TrendResponse {
    fn from(t: TrendAnalysis) -> Self {
        Self {
            location: TrendLocationDto {
                address: t.location.address,
                distance_m: t.location.distance_m,
            },
            data: t
                .data
                .into_iter()
                .map(|p| TrendPointDto {
                    year: p.year,
                    price_per_sqm: p.price_per_sqm,
                })
                .collect(),
            cagr: t.cagr,
            direction: t.direction.as_str().to_string(),
        }
    }
}
