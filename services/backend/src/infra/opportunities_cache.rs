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
//! Data types [`CachedOpportunitiesResponse`] and [`OpportunitiesCacheKey`]
//! live in the domain layer (`domain::entity` and `domain::value_object`
//! respectively). This module contains only the [`OpportunitiesCache`]
//! wrapper, which is an infra concern.

use std::sync::Arc;
use std::time::Duration;

use moka::future::Cache;

use crate::domain::constants::{OPPORTUNITY_CACHE_MAX_ENTRIES, OPPORTUNITY_CACHE_TTL_SECS};
use crate::domain::entity::CachedOpportunitiesResponse;
use crate::domain::value_object::OpportunitiesCacheKey;

/// 60-second in-memory cache for opportunities responses.
#[derive(Clone)]
pub(crate) struct OpportunitiesCache {
    inner: Cache<OpportunitiesCacheKey, Arc<CachedOpportunitiesResponse>>,
}

impl OpportunitiesCache {
    pub(crate) fn new() -> Self {
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
    pub(crate) async fn get_or_compute<F, Fut>(
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
