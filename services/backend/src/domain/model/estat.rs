//! Domain model types for e-Stat population and vacancy data.
//!
//! These types are produced by [`PopulationRepository`](crate::domain::repository::PopulationRepository)
//! and [`VacancyRepository`](crate::domain::repository::VacancyRepository),
//! and consumed by the corresponding usecases and handler response DTOs.

use crate::domain::model::primitives::{AreaName, CityCode};

/// Municipality-level population record sourced from the national census
/// (`国勢調査`) via e-Stat.
///
/// Each row corresponds to one city/ward with pivoted population breakdowns
/// from `mv_population_summary`.
#[derive(Debug, Clone)]
pub struct PopulationSummary {
    /// JIS X 0402 5-digit municipality code (e.g. `"13104"` for 新宿区).
    pub city_code: CityCode,
    /// Municipality name in Japanese (e.g. `"新宿区"`).
    pub city_name: AreaName,
    /// Total population (`category = '0010'`).
    pub population: i32,
    /// Male population (`category = '0020'`). `None` when not reported.
    pub male: Option<i32>,
    /// Female population (`category = '0030'`). `None` when not reported.
    pub female: Option<i32>,
    /// Number of households (`category = '0040'`). `None` when not reported.
    pub households: Option<i32>,
    /// Census year (e.g. `2020`).
    pub census_year: i16,
}

/// Municipality-level housing vacancy record sourced from the housing and
/// land survey (`住宅・土地統計調査`) via e-Stat.
///
/// Rows come from `mv_vacancy_summary` joined with `admin_boundaries`.
#[derive(Debug, Clone)]
pub struct VacancySummary {
    /// JIS X 0402 5-digit municipality code.
    pub city_code: CityCode,
    /// Municipality name in Japanese.
    pub city_name: AreaName,
    /// Number of vacant housing units.
    pub vacancy_count: i32,
    /// Total housing stock. `None` when not yet populated.
    pub total_houses: Option<i32>,
    /// Vacancy rate as a percentage rounded to one decimal place.
    /// `None` when `total_houses` is `None` or zero.
    pub vacancy_rate_pct: Option<f64>,
    /// Survey year (e.g. `2023`).
    pub survey_year: i16,
}
