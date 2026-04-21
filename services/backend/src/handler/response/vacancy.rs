//! Response DTOs for `GET /api/v1/vacancy`.

use serde::Serialize;

use crate::domain::model::VacancySummary;

/// Single municipality vacancy record returned in the response array.
#[derive(Debug, Serialize)]
pub struct VacancyResponse {
    /// 5-digit JIS X 0402 municipality code (e.g. `"13104"` for 新宿区).
    pub city_code: String,
    /// Municipality name in Japanese (e.g. `"新宿区"`).
    pub city_name: String,
    /// Number of vacant housing units.
    pub vacancy_count: i32,
    /// Total housing stock. `null` when not yet populated.
    pub total_houses: Option<i32>,
    /// Vacancy rate as a percentage rounded to one decimal place.
    /// `null` when `total_houses` is absent or zero.
    pub vacancy_rate_pct: Option<f64>,
    /// Survey year (e.g. `2023`).
    pub survey_year: i16,
}

impl From<VacancySummary> for VacancyResponse {
    fn from(s: VacancySummary) -> Self {
        Self {
            city_code: s.city_code.as_str().to_string(),
            city_name: s.city_name.as_str().to_string(),
            vacancy_count: s.vacancy_count,
            total_houses: s.total_houses,
            vacancy_rate_pct: s.vacancy_rate_pct,
            survey_year: s.survey_year,
        }
    }
}
