//! Shared query execution helpers for PostgreSQL repositories.

use std::future::Future;
use std::time::Duration;

use crate::domain::entity::{GeoFeature, LayerResult};
use crate::domain::error::DomainError;
use crate::infra::map_db_err::map_db_err;

/// Execute a sqlx query with a timeout, mapping both timeout and DB errors.
///
/// The `.inspect()` / `.inspect_err()` calls for tracing stay on the caller
/// side so each repository can log its own structured fields.
pub(crate) async fn run_query<T, Fut>(
    timeout_dur: Duration,
    label: &'static str,
    fut: Fut,
) -> Result<T, DomainError>
where
    Fut: Future<Output = Result<T, sqlx::Error>>,
{
    tokio::time::timeout(timeout_dur, fut)
        .await
        .map_err(|_| DomainError::Timeout(label.into()))?
        .map_err(map_db_err)
}

/// Apply the N+1 truncation pattern: receive `limit + 1` rows, check whether
/// more exist, then return at most `limit` features together with the flag.
pub(crate) fn apply_limit(mut features: Vec<GeoFeature>, limit: i64) -> LayerResult {
    let truncated = features.len() > limit as usize;
    if truncated {
        features.truncate(limit as usize);
    }
    LayerResult {
        features,
        truncated,
        limit,
    }
}
