//! Response DTOs for `GET /api/v1/population`.

use serde::Serialize;

use crate::domain::model::PopulationSummary;

/// Single municipality population record returned in the response array.
#[derive(Debug, Serialize)]
pub struct PopulationResponse {
    /// 5-digit JIS X 0402 municipality code (e.g. `"13104"` for 新宿区).
    pub city_code: String,
    /// Municipality name in Japanese (e.g. `"新宿区"`).
    pub city_name: String,
    /// Total population.
    pub population: i32,
    /// Male population. `null` when not reported.
    pub male: Option<i32>,
    /// Female population. `null` when not reported.
    pub female: Option<i32>,
    /// Number of households. `null` when not reported.
    pub households: Option<i32>,
    /// Census year (e.g. `2020`).
    pub census_year: i16,
}

impl From<PopulationSummary> for PopulationResponse {
    fn from(s: PopulationSummary) -> Self {
        Self {
            city_code: s.city_code.as_str().to_string(),
            city_name: s.city_name.as_str().to_string(),
            population: s.population,
            male: s.male,
            female: s.female,
            households: s.households,
            census_year: s.census_year,
        }
    }
}
