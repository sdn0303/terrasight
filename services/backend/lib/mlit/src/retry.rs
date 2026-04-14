//! Shared HTTP retry logic for MLIT API clients.
//!
//! This module is `pub(crate)` — it is an implementation detail not exposed
//! in the public API. All API clients delegate their HTTP calls to
//! [`request_with_retry`], which handles rate-limit back-off uniformly.

use std::time::Duration;

use crate::error::MlitError;

const MAX_RETRIES: u32 = 3;
const RETRY_BACKOFF_BASE: u64 = 2;

/// Execute an HTTP GET with exponential backoff retry on rate-limit and
/// transient transport failures.
///
/// Retries up to [`MAX_RETRIES`] (3) times. Back-off delays are
/// `RETRY_BACKOFF_BASE ^ attempt` seconds: 1 s → 2 s → 4 s.
///
/// | Condition | Behaviour |
/// |---|---|
/// | HTTP 429 (rate limited) | Retry with back-off; error after all attempts |
/// | Non-2xx (other) | Return [`MlitError::Api`] immediately (no retry) |
/// | Transport error | Retry with back-off; error after all attempts |
/// | 2xx | Return the [`reqwest::Response`] to the caller |
///
/// # Parameters
///
/// - `http`: Shared `reqwest` client (caller controls timeout configuration).
/// - `url`: Full request URL excluding query parameters.
/// - `params`: Query parameters appended to the URL.
/// - `auth_header`: Optional `(header_name, header_value)` for API key auth.
/// - `context`: Short label used in `tracing` log fields (e.g., `"reinfolib"`).
///
/// # Errors
///
/// Returns [`MlitError::RateLimited`] when HTTP 429 persists across all retry
/// attempts. Returns [`MlitError::Api`] for non-retryable non-2xx responses.
/// Returns [`MlitError::Http`] when all retry attempts fail with a transport
/// error.
pub(crate) async fn request_with_retry(
    http: &reqwest::Client,
    url: &str,
    params: &[(&str, &str)],
    auth_header: Option<(&'static str, &str)>,
    context: &'static str,
) -> Result<reqwest::Response, MlitError> {
    for attempt in 0..MAX_RETRIES {
        let mut req = http.get(url).query(params);
        if let Some((key, val)) = auth_header {
            req = req.header(key, val);
        }
        let result = req.send().await;

        match result {
            Ok(resp) if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS => {
                if attempt < MAX_RETRIES - 1 {
                    let delay = Duration::from_secs(RETRY_BACKOFF_BASE.pow(attempt));
                    tracing::warn!(attempt = attempt + 1, delay_secs = ?delay, context, "rate limited, retrying");
                    tokio::time::sleep(delay).await;
                    continue;
                }
                return Err(MlitError::RateLimited {
                    retry_after_secs: RETRY_BACKOFF_BASE.pow(attempt),
                });
            }
            Ok(resp) if !resp.status().is_success() => {
                let status = resp.status().as_u16();
                let message = resp
                    .text()
                    .await
                    .unwrap_or_else(|_| "<unreadable body>".into());
                return Err(MlitError::Api { status, message });
            }
            Ok(resp) => return Ok(resp),
            Err(e) => {
                if attempt < MAX_RETRIES - 1 {
                    let delay = Duration::from_secs(RETRY_BACKOFF_BASE.pow(attempt));
                    tracing::warn!(attempt = attempt + 1, error = %e, context, "request failed, retrying");
                    tokio::time::sleep(delay).await;
                    continue;
                }
                return Err(MlitError::Http(e));
            }
        }
    }
    unreachable!("retry loop always returns before exhausting MAX_RETRIES")
}
