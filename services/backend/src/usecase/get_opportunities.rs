//! `GET /api/v1/opportunities` usecase.
//!
//! Fetches a page of [`OpportunityRecord`]s from
//! [`LandPriceRepository::find_for_opportunities`], enriches each one with
//! a TLS score + risk level + signal by running
//! [`ComputeTlsUsecase::execute`] with bounded concurrency, and caches
//! the resulting page for `OPPORTUNITY_CACHE_TTL_SECS` seconds.
//!
//! ## Layering
//!
//! - The DB fetch happens BEFORE the cache check so database errors always
//!   propagate (cache only holds successes).
//! - TLS enrichment is driven by `futures::stream::buffer_unordered` so at
//!   most `OPPORTUNITY_TLS_CONCURRENCY` concurrent compute_tls calls hit
//!   the pool at once.
//! - Individual TLS compute failures are logged and skipped, not fatal —
//!   the response simply drops the affected record rather than failing
//!   the whole request.
//! - The whole pipeline is wrapped in `tokio::time::timeout` so a slow
//!   cache miss cannot hold an HTTP worker past `OPPORTUNITY_TIMEOUT_SECS`.

use std::sync::Arc;
use std::time::Duration;

use futures::stream::{self, StreamExt};
use tokio::time::timeout;

use crate::domain::constants::{OPPORTUNITY_TIMEOUT_SECS, OPPORTUNITY_TLS_CONCURRENCY};
use crate::domain::entity::{Opportunity, OpportunityRecord, Percent};
use crate::domain::error::DomainError;
use crate::domain::repository::{LandPriceRepository, TrendRepository};
use crate::domain::value_object::{OpportunitySignal, RiskLevel, TlsScore};
use crate::handler::request::OpportunitiesFilters;
use crate::infra::opportunities_cache::{OpportunitiesCache, OpportunitiesCacheKey};
use crate::usecase::compute_tls::ComputeTlsUsecase;

/// Re-export the shared cache-response type so that callers (handler,
/// response DTO) can import it from a single place regardless of whether
/// they came in via the F2 placeholder or the F5 definition.
pub use crate::infra::opportunities_cache::CachedOpportunitiesResponse;

pub struct GetOpportunitiesUsecase {
    land_repo: Arc<dyn LandPriceRepository>,
    #[allow(dead_code)]
    trend_repo: Arc<dyn TrendRepository>,
    compute_tls: Arc<ComputeTlsUsecase>,
    cache: Arc<OpportunitiesCache>,
}

impl GetOpportunitiesUsecase {
    pub fn new(
        land_repo: Arc<dyn LandPriceRepository>,
        trend_repo: Arc<dyn TrendRepository>,
        compute_tls: Arc<ComputeTlsUsecase>,
        cache: Arc<OpportunitiesCache>,
    ) -> Self {
        Self {
            land_repo,
            trend_repo,
            compute_tls,
            cache,
        }
    }

    #[tracing::instrument(skip(self), fields(usecase = "get_opportunities"))]
    pub async fn execute(
        &self,
        filters: OpportunitiesFilters,
    ) -> Result<Arc<CachedOpportunitiesResponse>, DomainError> {
        // Fetch BEFORE the cache so DB errors propagate and only successful
        // results get cached.
        let records = self
            .land_repo
            .find_for_opportunities(
                &filters.bbox,
                filters.limit,
                filters.offset,
                filters.price_range,
                &filters.zones,
            )
            .await
            .inspect_err(|e| tracing::warn!(error = %e, "opportunities fetch failed"))?;

        let key = Self::build_cache_key(&filters);
        let filters_loader = filters.clone();
        let compute_tls = self.compute_tls.clone();

        timeout(
            Duration::from_secs(OPPORTUNITY_TIMEOUT_SECS),
            self.cache.get_or_compute(key, move || async move {
                Self::enrich(records, filters_loader, compute_tls).await
            }),
        )
        .await
        .map_err(|_| DomainError::Timeout("opportunities".into()))
    }

    async fn enrich(
        records: Vec<OpportunityRecord>,
        filters: OpportunitiesFilters,
        compute_tls: Arc<ComputeTlsUsecase>,
    ) -> Arc<CachedOpportunitiesResponse> {
        let items: Vec<Opportunity> = stream::iter(records.into_iter())
            .map(|record| {
                let compute_tls = compute_tls.clone();
                let preset = filters.preset;
                async move {
                    let tls_output = compute_tls
                        .execute(&record.coord, preset)
                        .await
                        .inspect_err(
                            |e| tracing::warn!(id = record.id, error = %e, "TLS compute failed"),
                        )
                        .ok()?;
                    let tls = TlsScore::from_f64_clamped(tls_output.score);
                    let risk = RiskLevel::from_disaster_score(tls_output.axes.disaster.score);
                    Some(Opportunity {
                        record,
                        tls,
                        risk,
                        signal: OpportunitySignal::derive(tls, risk),
                        trend_pct: Percent::zero(),
                        station: None,
                    })
                }
            })
            .buffer_unordered(OPPORTUNITY_TLS_CONCURRENCY)
            .filter_map(|opt| async move { opt })
            .filter(|op| {
                let keep = filters
                    .tls_min
                    .is_none_or(|min| op.tls.value() >= min.value())
                    && filters.risk_max.is_none_or(|max| op.risk <= max);
                async move { keep }
            })
            .collect()
            .await;

        let total = items.len();
        Arc::new(CachedOpportunitiesResponse {
            items,
            total,
            truncated: false,
        })
    }

    fn build_cache_key(filters: &OpportunitiesFilters) -> OpportunitiesCacheKey {
        let to_microdeg = |v: f64| (v * 1_000_000.0) as i64;
        OpportunitiesCacheKey {
            bbox_microdeg: (
                to_microdeg(filters.bbox.west()),
                to_microdeg(filters.bbox.south()),
                to_microdeg(filters.bbox.east()),
                to_microdeg(filters.bbox.north()),
            ),
            limit: filters.limit.get(),
            offset: filters.offset.get(),
            tls_min: filters.tls_min.map(|s| s.value()),
            risk_max: filters.risk_max,
            zones: filters
                .zones
                .iter()
                .map(|z| z.as_str().to_string())
                .collect(),
            station_max: filters.station_max.map(|m| m.value()),
            price_range: filters.price_range.map(|(lo, hi)| (lo.value(), hi.value())),
            preset: filters.preset,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::{
        Address, BuildingCoverageRatio, FloorAreaRatio, PricePerSqm, ZoneCode,
    };
    use crate::domain::repository::mock::{
        MockLandPriceRepository, MockTlsRepository, MockTrendRepository,
    };
    use crate::domain::scoring::tls::WeightPreset;
    use crate::domain::value_object::{BBox, Coord, OpportunityLimit, OpportunityOffset, Year};

    fn sample_filters() -> OpportunitiesFilters {
        OpportunitiesFilters {
            bbox: BBox::new(35.65, 139.70, 35.70, 139.80).unwrap(),
            limit: OpportunityLimit::clamped(25),
            offset: OpportunityOffset::new(0),
            tls_min: None,
            risk_max: None,
            zones: Vec::new(),
            station_max: None,
            price_range: None,
            preset: WeightPreset::Balance,
        }
    }

    fn sample_record(id: i64) -> OpportunityRecord {
        OpportunityRecord {
            id,
            coord: Coord::new(35.689, 139.693).unwrap(),
            address: Address::parse("東京都新宿区西新宿1-1").unwrap(),
            zone: ZoneCode::parse("商業地域").unwrap(),
            building_coverage_ratio: BuildingCoverageRatio::new(80).unwrap(),
            floor_area_ratio: FloorAreaRatio::new(800).unwrap(),
            price_per_sqm: PricePerSqm::new(1_500_000).unwrap(),
            year: Year::new(2024).unwrap(),
        }
    }

    #[tokio::test]
    async fn execute_propagates_db_error() {
        let land_repo = Arc::new(
            MockLandPriceRepository::new()
                .with_find_for_opportunities(Err(DomainError::Database("boom".into()))),
        );
        let trend_repo: Arc<dyn TrendRepository> = Arc::new(MockTrendRepository::new());
        let tls_repo: Arc<dyn crate::domain::repository::TlsRepository> =
            Arc::new(MockTlsRepository::new());
        let compute_tls = Arc::new(ComputeTlsUsecase::new(tls_repo, None));
        let cache = Arc::new(OpportunitiesCache::new());

        let usecase = GetOpportunitiesUsecase::new(land_repo, trend_repo, compute_tls, cache);
        let result = usecase.execute(sample_filters()).await;

        assert!(matches!(result, Err(DomainError::Database(_))));
    }

    #[tokio::test]
    async fn execute_returns_empty_response_for_no_records() {
        let land_repo =
            Arc::new(MockLandPriceRepository::new().with_find_for_opportunities(Ok(Vec::new())));
        let trend_repo: Arc<dyn TrendRepository> = Arc::new(MockTrendRepository::new());
        let tls_repo: Arc<dyn crate::domain::repository::TlsRepository> =
            Arc::new(MockTlsRepository::new());
        let compute_tls = Arc::new(ComputeTlsUsecase::new(tls_repo, None));
        let cache = Arc::new(OpportunitiesCache::new());

        let usecase = GetOpportunitiesUsecase::new(land_repo, trend_repo, compute_tls, cache);
        let response = usecase.execute(sample_filters()).await.unwrap();

        assert_eq!(response.items.len(), 0);
        assert_eq!(response.total, 0);
        assert!(!response.truncated);
    }

    #[test]
    fn cache_key_is_invariant_under_bbox_jitter() {
        let mut filters_a = sample_filters();
        filters_a.bbox = BBox::new(35.65, 139.70, 35.70, 139.80).unwrap();

        let mut filters_b = sample_filters();
        // Add sub-micro-degree jitter that should quantize away
        filters_b.bbox = BBox::new(35.650000_1, 139.700000_1, 35.70, 139.80).unwrap();

        let key_a = GetOpportunitiesUsecase::build_cache_key(&filters_a);
        let key_b = GetOpportunitiesUsecase::build_cache_key(&filters_b);

        assert_eq!(key_a, key_b);
    }

    #[test]
    fn cache_key_differs_on_preset_change() {
        let filters_a = sample_filters();
        let mut filters_b = sample_filters();
        filters_b.preset = WeightPreset::Investment;

        let key_a = GetOpportunitiesUsecase::build_cache_key(&filters_a);
        let key_b = GetOpportunitiesUsecase::build_cache_key(&filters_b);

        assert_ne!(key_a, key_b);
    }

    #[test]
    fn sample_record_is_valid() {
        // Guardrail: if this constructor starts failing, the other async
        // tests will hang waiting on compute_tls without an obvious error.
        let _record = sample_record(1);
    }
}
