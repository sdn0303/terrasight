//! In-memory TTL cache for the `/api/v1/opportunities` endpoint.
//!
//! Wraps a [`moka::future::Cache`] keyed by [`OpportunitiesCacheKey`] —
//! a fingerprint of the validated filter set — so that two requests
//! with the same filters within `OPPORTUNITY_CACHE_TTL_SECS` share a
//! single TLS enrichment pass.
//!
//! The cached value is a [`CachedOpportunitiesResponse`] holding the
//! full TLS-enriched + `tls_min`/`risk_max`-filtered record pool. User
//! pagination (`limit`/`offset`) is applied **after** cache retrieval
//! by the handler so that every paginated view into the same filter
//! set hits the same cache slot.
//!
//! `CachedOpportunitiesResponse` lives in this module (not in
//! [`crate::usecase::get_opportunities`]) because the cache owns the
//! shape of the values it stores. The usecase module re-exports it via
//! `pub use` for ergonomics.

use std::sync::Arc;
use std::time::Duration;

use moka::future::Cache;

use crate::domain::constants::{OPPORTUNITY_CACHE_MAX_ENTRIES, OPPORTUNITY_CACHE_TTL_SECS};
use crate::domain::entity::Opportunity;
use crate::domain::scoring::tls::WeightPreset;
use crate::domain::value_object::RiskLevel;

/// TLS-enriched + filtered opportunities pool for a single cache slot.
///
/// Pagination (`limit`/`offset`) is intentionally **not** represented
/// here — the handler applies it after retrieving this value from the
/// cache, so a single cached pool serves every page of the same filter
/// set.
///
/// The `total` field reflects the number of records that survived TLS
/// enrichment + `tls_min`/`risk_max` post-filtering, not the raw DB
/// row count.
#[derive(Debug, Clone, Default)]
pub struct CachedOpportunitiesResponse {
    pub items: Vec<Opportunity>,
    pub total: usize,
}

/// Fingerprint of a validated opportunities request, excluding the
/// pagination parameters.
///
/// - `bbox_microdeg` quantizes the float bounding box to micro-degrees
///   so sub-pixel jitter maps to the same cache slot.
/// - `zones` is stored sorted + deduped (canonicalized by
///   [`crate::usecase::get_opportunities::GetOpportunitiesUsecase::build_cache_key`])
///   so that `zones=A,B` and `zones=B,A,A` hit the same slot.
/// - `limit`/`offset` are absent by design — they are applied after
///   cache retrieval.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OpportunitiesCacheKey {
    pub bbox_microdeg: (i64, i64, i64, i64),
    pub tls_min: Option<u8>,
    pub risk_max: Option<RiskLevel>,
    pub zones: Vec<String>,
    pub station_max: Option<u32>,
    pub price_range: Option<(i64, i64)>,
    pub preset: WeightPreset,
}

/// 60-second in-memory cache for opportunities responses.
#[derive(Clone)]
pub struct OpportunitiesCache {
    inner: Cache<OpportunitiesCacheKey, Arc<CachedOpportunitiesResponse>>,
}

impl OpportunitiesCache {
    pub fn new() -> Self {
        Self {
            inner: Cache::builder()
                .max_capacity(OPPORTUNITY_CACHE_MAX_ENTRIES)
                .time_to_live(Duration::from_secs(OPPORTUNITY_CACHE_TTL_SECS))
                .build(),
        }
    }

    /// Return the cached entry for `key`, or run `loader` to populate it.
    ///
    /// Uses `moka::Cache::get_with` so concurrent callers for the same key
    /// share a single loader invocation.
    pub async fn get_or_compute<F, Fut>(
        &self,
        key: OpportunitiesCacheKey,
        loader: F,
    ) -> Arc<CachedOpportunitiesResponse>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Arc<CachedOpportunitiesResponse>>,
    {
        self.inner.get_with(key, loader()).await
    }
}

impl Default for OpportunitiesCache {
    fn default() -> Self {
        Self::new()
    }
}
