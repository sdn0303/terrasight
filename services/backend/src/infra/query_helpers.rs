//! Shared query execution helpers for PostgreSQL repositories.
//!
//! Provides two utilities used by every `pg_*_repository`:
//!
//! - [`run_query`] — wraps any `sqlx` future with a `tokio::time::timeout` and
//!   maps both timeout and database errors to [`DomainError`].
//! - [`apply_limit`] — implements the N+1 truncation pattern for spatial feature
//!   queries, avoiding a separate `COUNT(*)` round-trip.

use std::future::Future;
use std::time::Duration;

use crate::domain::error::DomainError;
use crate::domain::model::{GeoFeature, LayerResult};
use crate::infra::map_db_err::map_db_err;

/// Execute a sqlx query future with a deadline, mapping errors to [`DomainError`].
///
/// The `.inspect()` / `.inspect_err()` tracing calls belong on the caller side
/// so each repository can attach its own structured fields without this helper
/// becoming a tracing hot-spot.
///
/// # Errors
///
/// Returns [`DomainError::Timeout`] if `fut` does not complete within
/// `timeout_dur`. Returns [`DomainError::Database`] for any `sqlx::Error`.
///
/// # Examples
///
/// ```ignore
/// // run_query is pub(crate); shown here for illustration only.
/// // Repositories call it directly within the crate:
/// //
/// //   let row = run_query(TIMEOUT, "my_query", sqlx_future).await?;
/// ```
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

/// Apply the N+1 truncation pattern to a feature list.
///
/// Repositories request `limit + 1` rows. If the result set is larger than
/// `limit`, at least one additional row exists — `truncated` is set to `true`
/// and the excess row is dropped. This avoids a separate `COUNT(*)` query
/// while still letting the frontend know the result was capped.
///
/// # Examples
///
/// ```ignore
/// // apply_limit is pub(crate). Inside a repository:
/// //
/// //   let rows = run_query(TIMEOUT, "q", query.bind(limit + 1).fetch_all(&pool)).await?;
/// //   Ok(apply_limit(rows.into_iter().map(GeoFeature::from).collect(), limit))
/// ```
pub(crate) fn apply_limit(mut features: Vec<GeoFeature>, limit: i64) -> LayerResult {
    let limit_usize = usize::try_from(limit).expect("INVARIANT: limit is positive");
    let truncated = features.len() > limit_usize;
    if truncated {
        features.truncate(limit_usize);
    }
    LayerResult {
        features,
        truncated,
        limit,
    }
}
