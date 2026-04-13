//! Prometheus metrics: recorder initialization, scrape endpoint, and re-exported macros.
//!
//! This module wires up the [`metrics_exporter_prometheus`] recorder as the
//! global [`metrics`] backend and spawns a background Tokio task that serves
//! the standard `/metrics` scrape endpoint.
//!
//! ## Submodules
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`names`] | Metric name constants following `{subsystem}.{resource}.{action}` |
//! | [`tags`] | Label name constants shared across all emitting crates |
//!
//! ## Quick start
//!
//! ```rust,ignore
//! use terrasight_server::metrics::{self, names, tags};
//!
//! // At startup
//! metrics::init_prometheus("0.0.0.0:9090".parse().unwrap()).await?;
//!
//! // In a handler
//! metrics::counter!(names::API_REQUEST_TOTAL, tags::METHOD => "GET", tags::STATUS => "200")
//!     .increment(1);
//! ```
//!
//! ## Re-exports
//!
//! [`counter!`], [`gauge!`], and [`histogram!`] from the [`metrics`] crate are
//! re-exported so consuming crates do not need `metrics` as a direct dependency.

pub mod names;
pub mod tags;

use std::net::SocketAddr;

/// Error returned when the Prometheus metrics recorder cannot be initialized.
#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    /// The global `metrics` recorder slot is already occupied by another recorder.
    #[error("Failed to install global recorder: {0:?}")]
    SetRecorder(metrics::SetRecorderError<metrics_exporter_prometheus::PrometheusRecorder>),
    /// The OS rejected the TCP bind for the scrape endpoint.
    #[error("Failed to bind metrics listener on {addr}: {source}")]
    Bind {
        addr: SocketAddr,
        source: std::io::Error,
    },
}

impl From<metrics::SetRecorderError<metrics_exporter_prometheus::PrometheusRecorder>>
    for MetricsError
{
    fn from(
        err: metrics::SetRecorderError<metrics_exporter_prometheus::PrometheusRecorder>,
    ) -> Self {
        Self::SetRecorder(err)
    }
}

/// Start a Prometheus-compatible `/metrics` HTTP endpoint.
///
/// Installs the global `metrics` recorder and spawns a background Tokio task
/// that serves the Prometheus scrape endpoint on `addr`.
///
/// # Arguments
///
/// * `addr` — Socket address to bind (e.g., `0.0.0.0:9090`).
///
/// # Errors
///
/// Returns [`MetricsError`] if the recorder is already set or the TCP bind fails.
///
/// # Example
///
/// ```rust,ignore
/// terrasight_server::metrics::init_prometheus("0.0.0.0:9090".parse().unwrap()).await?;
/// ```
pub async fn init_prometheus(addr: SocketAddr) -> Result<(), MetricsError> {
    let builder = metrics_exporter_prometheus::PrometheusBuilder::new();
    let (recorder, exporter) = builder
        .with_http_listener(addr)
        .build()
        .expect("INVARIANT: PrometheusBuilder::build is infallible with default settings");

    metrics::set_global_recorder(recorder)?;

    // Spawn the exporter as a background task — it serves /metrics until the
    // runtime shuts down.
    tokio::spawn(async move {
        if let Err(e) = exporter.await {
            tracing::error!(error = ?e, "Prometheus exporter failed");
        }
    });

    tracing::info!(%addr, "Prometheus metrics endpoint started");
    Ok(())
}

// Re-export metrics macros so consumers don't need `metrics` as a direct dependency.
pub use metrics::{counter, gauge, histogram};
