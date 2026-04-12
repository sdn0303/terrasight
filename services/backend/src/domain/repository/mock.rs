//! Handwritten mock implementations of every repository trait in
//! [`crate::domain::repository`], used by colocated usecase unit tests.
//!
//! Each mock holds a `Mutex<Vec<Result<T, DomainError>>>` queue per trait
//! method. `.with_<method>(result)` appends responses in the order the
//! test expects them to be consumed. Calls past the end of the queue
//! panic with a descriptive message so missing setup is surfaced loudly
//! during `cargo test` rather than returning silent `Default`s.
//!
//! Mocks live behind `#[cfg(test)]` so they never ship in a release
//! binary; they stay in the domain layer because they implement
//! domain-owned traits and need the same visibility as the traits
//! themselves.

#![cfg(test)]

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;

use crate::domain::entity::{
    AdminAreaStats, FacilityStats, LandPriceStats, LayerResult, MedicalStats, OpportunityRecord,
    PricePerSqm, PriceRecord, RiskStats, SchoolStats, TrendLocation, TrendPoint, ZScoreResult,
    ZoneCode,
};
use crate::domain::error::DomainError;
use crate::domain::repository::{
    AdminAreaStatsRepository, HealthRepository, LandPriceRepository, LayerRepository,
    StatsRepository, TlsRepository, TrendRepository,
};
use crate::domain::value_object::{
    AreaCode, BBox, Coord, LayerType, PrefCode, Year, YearsLookback, ZoomLevel,
};

/// Pop the next queued response or panic if the queue is empty.
///
/// Panicking is intentional: it means a test forgot to register a
/// result for the call under inspection.
fn pop<T>(
    queue: &Mutex<Vec<Result<T, DomainError>>>,
    method: &'static str,
) -> Result<T, DomainError> {
    let mut guard = queue.lock().expect("mock queue poisoned");
    if guard.is_empty() {
        panic!("mock queue for `{method}` is empty — add `.with_{method}(...)` in the test setup");
    }
    guard.remove(0)
}

// ─── LayerRepository ──────────────────────────────────────────────────────────

#[derive(Default)]
pub struct MockLayerRepository {
    find_layer: Mutex<Vec<Result<LayerResult, DomainError>>>,
}

impl MockLayerRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_find_layer(self, result: Result<LayerResult, DomainError>) -> Self {
        self.find_layer
            .lock()
            .expect("mock queue poisoned")
            .push(result);
        self
    }
}

#[async_trait]
impl LayerRepository for MockLayerRepository {
    async fn find_layer(
        &self,
        _layer: LayerType,
        _bbox: &BBox,
        _zoom: ZoomLevel,
        _pref_code: Option<&PrefCode>,
    ) -> Result<LayerResult, DomainError> {
        pop(&self.find_layer, "find_layer")
    }
}

// ─── StatsRepository ──────────────────────────────────────────────────────────

#[derive(Default)]
pub struct MockStatsRepository {
    land_price: Mutex<Vec<Result<LandPriceStats, DomainError>>>,
    risk: Mutex<Vec<Result<RiskStats, DomainError>>>,
    facilities: Mutex<Vec<Result<FacilityStats, DomainError>>>,
    zoning: Mutex<Vec<Result<HashMap<String, f64>, DomainError>>>,
}

impl MockStatsRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_land_price(self, r: Result<LandPriceStats, DomainError>) -> Self {
        self.land_price.lock().unwrap().push(r);
        self
    }

    pub fn with_risk(self, r: Result<RiskStats, DomainError>) -> Self {
        self.risk.lock().unwrap().push(r);
        self
    }

    pub fn with_facilities(self, r: Result<FacilityStats, DomainError>) -> Self {
        self.facilities.lock().unwrap().push(r);
        self
    }

    pub fn with_zoning_distribution(self, r: Result<HashMap<String, f64>, DomainError>) -> Self {
        self.zoning.lock().unwrap().push(r);
        self
    }
}

#[async_trait]
impl StatsRepository for MockStatsRepository {
    async fn calc_land_price_stats(
        &self,
        _bbox: &BBox,
        _pref_code: Option<&PrefCode>,
    ) -> Result<LandPriceStats, DomainError> {
        pop(&self.land_price, "land_price")
    }

    async fn calc_risk_stats(
        &self,
        _bbox: &BBox,
        _pref_code: Option<&PrefCode>,
    ) -> Result<RiskStats, DomainError> {
        pop(&self.risk, "risk")
    }

    async fn count_facilities(
        &self,
        _bbox: &BBox,
        _pref_code: Option<&PrefCode>,
    ) -> Result<FacilityStats, DomainError> {
        pop(&self.facilities, "facilities")
    }

    async fn calc_zoning_distribution(
        &self,
        _bbox: &BBox,
        _pref_code: Option<&PrefCode>,
    ) -> Result<HashMap<String, f64>, DomainError> {
        pop(&self.zoning, "zoning_distribution")
    }
}

// ─── TrendRepository ──────────────────────────────────────────────────────────

type TrendSnapshot = Option<(TrendLocation, Vec<TrendPoint>)>;

#[derive(Default)]
pub struct MockTrendRepository {
    find_trend: Mutex<Vec<Result<TrendSnapshot, DomainError>>>,
}

impl MockTrendRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_find_trend(self, r: Result<TrendSnapshot, DomainError>) -> Self {
        self.find_trend.lock().unwrap().push(r);
        self
    }
}

#[async_trait]
impl TrendRepository for MockTrendRepository {
    async fn find_trend(
        &self,
        _coord: Coord,
        _years: YearsLookback,
    ) -> Result<TrendSnapshot, DomainError> {
        pop(&self.find_trend, "find_trend")
    }
}

// ─── LandPriceRepository ──────────────────────────────────────────────────────

#[derive(Default)]
pub struct MockLandPriceRepository {
    find_by_year: Mutex<Vec<Result<LayerResult, DomainError>>>,
    find_all_years: Mutex<Vec<Result<LayerResult, DomainError>>>,
    find_for_opportunities: Mutex<Vec<Result<Vec<OpportunityRecord>, DomainError>>>,
}

impl MockLandPriceRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_find_by_year_and_bbox(self, r: Result<LayerResult, DomainError>) -> Self {
        self.find_by_year.lock().unwrap().push(r);
        self
    }

    pub fn with_find_all_years_by_bbox(self, r: Result<LayerResult, DomainError>) -> Self {
        self.find_all_years.lock().unwrap().push(r);
        self
    }

    pub fn with_find_for_opportunities(
        self,
        r: Result<Vec<OpportunityRecord>, DomainError>,
    ) -> Self {
        self.find_for_opportunities.lock().unwrap().push(r);
        self
    }
}

#[async_trait]
impl LandPriceRepository for MockLandPriceRepository {
    async fn find_by_year_and_bbox(
        &self,
        _year: Year,
        _bbox: &BBox,
        _zoom: ZoomLevel,
        _pref_code: Option<&PrefCode>,
    ) -> Result<LayerResult, DomainError> {
        pop(&self.find_by_year, "find_by_year_and_bbox")
    }

    async fn find_all_years_by_bbox(
        &self,
        _from_year: Year,
        _to_year: Year,
        _bbox: &BBox,
        _zoom: ZoomLevel,
        _pref_code: Option<&PrefCode>,
    ) -> Result<LayerResult, DomainError> {
        pop(&self.find_all_years, "find_all_years_by_bbox")
    }

    async fn find_for_opportunities(
        &self,
        _bbox: &BBox,
        _limit: u32,
        _offset: u32,
        _price_range: Option<(PricePerSqm, PricePerSqm)>,
        _zones: &[ZoneCode],
        _pref_code: Option<&PrefCode>,
    ) -> Result<Vec<OpportunityRecord>, DomainError> {
        pop(&self.find_for_opportunities, "find_for_opportunities")
    }
}

// ─── AdminAreaStatsRepository ────────────────────────────────────────────────

#[derive(Default)]
pub struct MockAdminAreaStatsRepository {
    get_area_stats: Mutex<Vec<Result<AdminAreaStats, DomainError>>>,
}

impl MockAdminAreaStatsRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_get_area_stats(self, r: Result<AdminAreaStats, DomainError>) -> Self {
        self.get_area_stats.lock().unwrap().push(r);
        self
    }
}

#[async_trait]
impl AdminAreaStatsRepository for MockAdminAreaStatsRepository {
    async fn get_area_stats(&self, _code: &AreaCode) -> Result<AdminAreaStats, DomainError> {
        pop(&self.get_area_stats, "get_area_stats")
    }
}

// ─── HealthRepository ─────────────────────────────────────────────────────────

#[derive(Default)]
pub struct MockHealthRepository {
    check: Mutex<Vec<bool>>,
}

impl MockHealthRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_check_connection(self, connected: bool) -> Self {
        self.check.lock().unwrap().push(connected);
        self
    }
}

#[async_trait]
impl HealthRepository for MockHealthRepository {
    async fn check_connection(&self) -> bool {
        let mut guard = self.check.lock().unwrap();
        if guard.is_empty() {
            panic!("mock queue for `check_connection` is empty");
        }
        guard.remove(0)
    }
}

// ─── TlsRepository ────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct MockTlsRepository {
    nearest_prices: Mutex<Vec<Result<Vec<PriceRecord>, DomainError>>>,
    flood_depth: Mutex<Vec<Result<Option<i32>, DomainError>>>,
    steep_slope: Mutex<Vec<Result<bool, DomainError>>>,
    schools: Mutex<Vec<Result<SchoolStats, DomainError>>>,
    medical: Mutex<Vec<Result<MedicalStats, DomainError>>>,
    zoning_far: Mutex<Vec<Result<Option<f64>, DomainError>>>,
    z_score: Mutex<Vec<Result<ZScoreResult, DomainError>>>,
    recent_tx: Mutex<Vec<Result<i64, DomainError>>>,
}

impl MockTlsRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_find_nearest_prices(self, r: Result<Vec<PriceRecord>, DomainError>) -> Self {
        self.nearest_prices.lock().unwrap().push(r);
        self
    }

    pub fn with_find_flood_depth_rank(self, r: Result<Option<i32>, DomainError>) -> Self {
        self.flood_depth.lock().unwrap().push(r);
        self
    }

    pub fn with_has_steep_slope_nearby(self, r: Result<bool, DomainError>) -> Self {
        self.steep_slope.lock().unwrap().push(r);
        self
    }

    pub fn with_find_schools_nearby(self, r: Result<SchoolStats, DomainError>) -> Self {
        self.schools.lock().unwrap().push(r);
        self
    }

    pub fn with_find_medical_nearby(self, r: Result<MedicalStats, DomainError>) -> Self {
        self.medical.lock().unwrap().push(r);
        self
    }

    pub fn with_find_zoning_far(self, r: Result<Option<f64>, DomainError>) -> Self {
        self.zoning_far.lock().unwrap().push(r);
        self
    }

    pub fn with_calc_price_z_score(self, r: Result<ZScoreResult, DomainError>) -> Self {
        self.z_score.lock().unwrap().push(r);
        self
    }

    pub fn with_count_recent_transactions(self, r: Result<i64, DomainError>) -> Self {
        self.recent_tx.lock().unwrap().push(r);
        self
    }
}

#[async_trait]
impl TlsRepository for MockTlsRepository {
    async fn find_nearest_prices(&self, _coord: &Coord) -> Result<Vec<PriceRecord>, DomainError> {
        pop(&self.nearest_prices, "find_nearest_prices")
    }

    async fn find_flood_depth_rank(&self, _coord: &Coord) -> Result<Option<i32>, DomainError> {
        pop(&self.flood_depth, "find_flood_depth_rank")
    }

    async fn has_steep_slope_nearby(&self, _coord: &Coord) -> Result<bool, DomainError> {
        pop(&self.steep_slope, "has_steep_slope_nearby")
    }

    async fn find_schools_nearby(&self, _coord: &Coord) -> Result<SchoolStats, DomainError> {
        pop(&self.schools, "find_schools_nearby")
    }

    async fn find_medical_nearby(&self, _coord: &Coord) -> Result<MedicalStats, DomainError> {
        pop(&self.medical, "find_medical_nearby")
    }

    async fn find_zoning_far(&self, _coord: &Coord) -> Result<Option<f64>, DomainError> {
        pop(&self.zoning_far, "find_zoning_far")
    }

    async fn calc_price_z_score(&self, _coord: &Coord) -> Result<ZScoreResult, DomainError> {
        pop(&self.z_score, "calc_price_z_score")
    }

    async fn count_recent_transactions(&self, _coord: &Coord) -> Result<i64, DomainError> {
        pop(&self.recent_tx, "count_recent_transactions")
    }
}
