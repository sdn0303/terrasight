pub mod names;
pub mod tags;

use std::net::SocketAddr;

/// Error returned when the Prometheus metrics recorder cannot be initialized.
#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    #[error("Failed to install global recorder: {0:?}")]
    SetRecorder(metrics::SetRecorderError<metrics_exporter_prometheus::PrometheusRecorder>),
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
/// realestate_telemetry::metrics::init_prometheus("0.0.0.0:9090".parse().unwrap()).await?;
/// ```
pub async fn init_prometheus(addr: SocketAddr) -> Result<(), MetricsError> {
    let builder = metrics_exporter_prometheus::PrometheusBuilder::new();
    let (recorder, exporter) = builder
        .with_http_listener(addr)
        .build()
        .expect("PrometheusBuilder::build is infallible with default settings");

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
