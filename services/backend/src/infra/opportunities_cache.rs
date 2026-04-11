//! In-memory TTL cache for the `/api/v1/opportunities` endpoint.
//!
//! Wraps a `moka::future::Cache` keyed by [`OpportunitiesCacheKey`] — a
//! fingerprint of the validated filter set — so that two requests with the
//! same filters within `OPPORTUNITY_CACHE_TTL_SECS` share a single TLS
//! enrichment pass.
//!
//! `CachedOpportunitiesResponse` is defined here as a placeholder in Phase
//! F2 and replaced in F5 with a `pub use` of the real type from
//! `usecase::get_opportunities`. The placeholder exists because the usecase
//! cannot be written before the cache it depends on is available.

use std::sync::Arc;
use std::time::Duration;

use moka::future::Cache;

use crate::domain::constants::{OPPORTUNITY_CACHE_MAX_ENTRIES, OPPORTUNITY_CACHE_TTL_SECS};
use crate::domain::entity::Opportunity;
use crate::domain::scoring::tls::WeightPreset;
use crate::domain::value_object::RiskLevel;

/// Placeholder shape for the cached opportunities response.
///
/// Defined here in F2 so the cache can be written before the usecase.
/// F5 replaces this with a `pub use` of the real type from
/// [`crate::usecase::get_opportunities`]. The fields are kept in sync
/// with the F5 definition so dependent code (handler/response,
/// OpportunitiesCache) compiles against either version.
#[derive(Debug, Clone, Default)]
pub struct CachedOpportunitiesResponse {
    pub items: Vec<Opportunity>,
    pub total: usize,
    pub truncated: bool,
}

/// Fingerprint of a validated opportunities request.
///
/// Floats are quantized to micro-degrees so that requests with
/// equivalent bounding boxes (up to floating-point noise) share a cache
/// slot.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OpportunitiesCacheKey {
    pub bbox_microdeg: (i64, i64, i64, i64),
    pub limit: u32,
    pub offset: u32,
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
