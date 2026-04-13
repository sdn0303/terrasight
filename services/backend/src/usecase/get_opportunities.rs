//! `GET /api/v1/opportunities` usecase.
//!
//! Fetches a fixed-size pool of [`OpportunityRecord`]s from
//! [`LandPriceRepository::find_for_opportunities`], enriches each one
//! with a TLS score + risk level + signal by running
//! [`ComputeTlsUsecase::execute`] with bounded concurrency, filters by
//! `tls_min`/`risk_max`, and caches the result pool for
//! `OPPORTUNITY_CACHE_TTL_SECS` seconds.
//!
//! ## Pagination
//!
//! User-facing `limit`/`offset` pagination is **not** applied inside
//! this usecase. The cached value is the full filtered pool; the
//! handler layer applies pagination via
//! [`crate::handler::response::OpportunitiesResponseDto::paginated`]
//! after cache retrieval. This means every paginated view into the
//! same filter set hits the same cache slot.
//!
//! Pagination is bounded by [`OPPORTUNITY_FETCH_POOL_SIZE`]: offset
//! values beyond the filtered pool size return empty results.
//!
//! ## Layering
//!
//! - The DB fetch happens BEFORE the cache check so database errors
//!   always propagate (cache only holds successes).
//! - TLS enrichment is driven by `futures::stream::buffer_unordered`
//!   so at most [`OPPORTUNITY_TLS_CONCURRENCY`] concurrent
//!   `compute_tls` calls hit the pool at once.
//! - Individual TLS compute failures are logged and skipped, not
//!   fatal — the pool simply drops the affected record.
//! - The whole pipeline is wrapped in `tokio::time::timeout` so a
//!   slow cache miss cannot hold an HTTP worker past
//!   [`OPPORTUNITY_TIMEOUT_SECS`].

use std::sync::Arc;
use std::time::Duration;

use futures::stream::{self, StreamExt};
use tokio::time::timeout;

use crate::domain::constants::{
    OPPORTUNITY_FETCH_POOL_SIZE, OPPORTUNITY_TIMEOUT_SECS, OPPORTUNITY_TLS_CONCURRENCY,
};
use crate::domain::entity::{CachedOpportunitiesResponse, Opportunity, OpportunityRecord, Percent};
use crate::domain::error::DomainError;
use crate::domain::repository::LandPriceRepository;
use crate::domain::value_object::{
    OpportunitiesCacheKey, OpportunitiesFilters, OpportunitySignal, RiskLevel, TlsScore,
};
use crate::infra::opportunities_cache::OpportunitiesCache;
use crate::usecase::compute_tls::ComputeTlsUsecase;

/// Usecase for `GET /api/v1/opportunities`.
pub(crate) struct GetOpportunitiesUsecase {
    land_repo: Arc<dyn LandPriceRepository>,
    compute_tls: Arc<ComputeTlsUsecase>,
    cache: Arc<OpportunitiesCache>,
}

impl GetOpportunitiesUsecase {
    /// Construct the usecase with its three dependencies.
    ///
    /// - `land_repo` — fetches raw opportunity records from PostGIS.
    /// - `compute_tls` — enriches each record with a TLS score (shared with `/api/v1/score`).
    /// - `cache` — in-memory TTL cache keyed by the filter fingerprint.
    pub(crate) fn new(
        land_repo: Arc<dyn LandPriceRepository>,
        compute_tls: Arc<ComputeTlsUsecase>,
        cache: Arc<OpportunitiesCache>,
    ) -> Self {
        Self {
            land_repo,
            compute_tls,
            cache,
        }
    }

    /// Fetch, enrich, and cache the full filtered opportunity pool.
    ///
    /// The returned [`CachedOpportunitiesResponse`] holds every record
    /// that survived TLS enrichment and `tls_min`/`risk_max` filtering.
    /// The handler applies `limit`/`offset` pagination to this pool after
    /// cache retrieval, so all paginated views of the same filter set share
    /// one cache slot.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] if the initial DB fetch fails, or
    /// [`DomainError::Timeout`] if the full enrichment pipeline exceeds
    /// [`OPPORTUNITY_TIMEOUT_SECS`](crate::domain::constants::OPPORTUNITY_TIMEOUT_SECS).
    #[tracing::instrument(skip(self), fields(usecase = "get_opportunities"))]
    pub(crate) async fn execute(
        &self,
        filters: OpportunitiesFilters,
    ) -> Result<Arc<CachedOpportunitiesResponse>, DomainError> {
        // Fetch BEFORE the cache so DB errors propagate and only
        // successful results get cached. Always request the full fetch
        // pool; user pagination is applied after cache retrieval by the handler.
        let records = self
            .land_repo
            .find_for_opportunities(
                &filters.bbox,
                OPPORTUNITY_FETCH_POOL_SIZE,
                filters.price_range,
                &filters.zones,
                filters.pref_code.as_ref(),
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
        Arc::new(CachedOpportunitiesResponse { items, total })
    }

    /// Build the cache key fingerprint for `filters`.
    ///
    /// `limit`/`offset` are intentionally excluded — pagination is
    /// post-cache, so all paginated views of the same filter set share
    /// a cache slot. `zones` is canonicalized (sorted + deduped) so
    /// `zones=A,B` and `zones=B,A,A` hit the same slot.
    fn build_cache_key(filters: &OpportunitiesFilters) -> OpportunitiesCacheKey {
        let to_microdeg = |v: f64| (v * 1_000_000.0) as i64;
        let mut zones: Vec<String> = filters
            .zones
            .iter()
            .map(|z| z.as_str().to_string())
            .collect();
        zones.sort();
        zones.dedup();

        OpportunitiesCacheKey {
            bbox_microdeg: (
                to_microdeg(filters.bbox.west()),
                to_microdeg(filters.bbox.south()),
                to_microdeg(filters.bbox.east()),
                to_microdeg(filters.bbox.north()),
            ),
            tls_min: filters.tls_min.map(|s| s.value()),
            risk_max: filters.risk_max,
            zones,
            station_max: filters.station_max.map(|m| m.value()),
            price_range: filters.price_range.map(|(lo, hi)| (lo.value(), hi.value())),
            preset: filters.preset,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::ZoneCode;
    use crate::domain::repository::mock::{MockLandPriceRepository, MockTlsRepository};
    use crate::domain::value_object::{BBox, OpportunityLimit, OpportunityOffset};
    use terrasight_domain::scoring::tls::WeightPreset;

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
            pref_code: None,
            cities: Vec::new(),
        }
    }

    fn make_usecase(land_repo: Arc<MockLandPriceRepository>) -> GetOpportunitiesUsecase {
        let tls_repo: Arc<dyn crate::domain::repository::TlsRepository> =
            Arc::new(MockTlsRepository::new());
        let compute_tls = Arc::new(ComputeTlsUsecase::new(tls_repo, None));
        let cache = Arc::new(OpportunitiesCache::new());
        GetOpportunitiesUsecase::new(land_repo, compute_tls, cache)
    }

    #[tokio::test]
    async fn execute_propagates_db_error() {
        let land_repo = Arc::new(
            MockLandPriceRepository::new()
                .with_find_for_opportunities(Err(DomainError::Database("boom".into()))),
        );
        let usecase = make_usecase(land_repo);
        let result = usecase.execute(sample_filters()).await;
        assert!(matches!(result, Err(DomainError::Database(_))));
    }

    #[tokio::test]
    async fn execute_returns_empty_pool_for_no_records() {
        let land_repo =
            Arc::new(MockLandPriceRepository::new().with_find_for_opportunities(Ok(Vec::new())));
        let usecase = make_usecase(land_repo);
        let cached = usecase.execute(sample_filters()).await.unwrap();

        assert_eq!(cached.items.len(), 0);
        assert_eq!(cached.total, 0);
    }

    #[test]
    fn cache_key_is_invariant_under_bbox_jitter() {
        let mut filters_a = sample_filters();
        filters_a.bbox = BBox::new(35.65, 139.70, 35.70, 139.80).unwrap();

        let mut filters_b = sample_filters();
        // Sub-micro-degree jitter should quantize away.
        filters_b.bbox = BBox::new(35.650000_1, 139.700000_1, 35.70, 139.80).unwrap();

        let key_a = GetOpportunitiesUsecase::build_cache_key(&filters_a);
        let key_b = GetOpportunitiesUsecase::build_cache_key(&filters_b);
        assert_eq!(key_a, key_b);
    }

    #[test]
    fn cache_key_is_invariant_under_zone_order_and_duplicates() {
        let mut filters_a = sample_filters();
        filters_a.zones = vec![
            ZoneCode::parse("商業地域").unwrap(),
            ZoneCode::parse("近隣商業地域").unwrap(),
        ];

        let mut filters_b = sample_filters();
        filters_b.zones = vec![
            ZoneCode::parse("近隣商業地域").unwrap(),
            ZoneCode::parse("商業地域").unwrap(),
            ZoneCode::parse("商業地域").unwrap(), // duplicate
        ];

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
    fn cache_key_is_invariant_under_limit_offset_change() {
        // Pagination is post-cache, so limit/offset must NOT contribute
        // to the cache key — otherwise we would cache per-page and lose
        // sharing across paginated views of the same filter set.
        let filters_a = sample_filters();
        let mut filters_b = sample_filters();
        filters_b.limit = OpportunityLimit::clamped(10);
        filters_b.offset = OpportunityOffset::new(25);

        let key_a = GetOpportunitiesUsecase::build_cache_key(&filters_a);
        let key_b = GetOpportunitiesUsecase::build_cache_key(&filters_b);
        assert_eq!(key_a, key_b);
    }
}
