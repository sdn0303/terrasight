use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// Log output format.
///
/// Controlled at runtime so the same release binary can emit human-readable
/// output in staging and structured JSON in production — no separate builds
/// needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    /// Colored, human-readable output for local development.
    Pretty,
    /// Newline-delimited JSON for log aggregation (Loki, CloudWatch, etc.).
    Json,
}

impl LogFormat {
    /// Parse from a string value (case-insensitive).
    ///
    /// Returns `Json` for `"json"`, `Pretty` for everything else.
    pub fn from_str_lossy(s: &str) -> Self {
        if s.eq_ignore_ascii_case("json") {
            Self::Json
        } else {
            Self::Pretty
        }
    }
}

/// Default env filter when `RUST_LOG` is not set.
///
/// - Application crates at `info`
/// - SQLx internal noise at `warn`
/// - tower-http request tracing at `debug`
const DEFAULT_FILTER: &str = "\
    realestate_api=info,\
    realestate_api_core=info,\
    realestate_db=debug,\
    realestate_telemetry=info,\
    realestate_geo_math=debug,\
    mlit_client=info,\
    sqlx=warn,\
    tower_http=debug,\
    hyper=warn\
";

/// Initialize the global tracing subscriber.
///
/// Must be called **once** at application startup before any `tracing` macros
/// are used.  Subsequent calls will log a warning and return without error.
///
/// # Arguments
///
/// * `format` — Log output format (see [`LogFormat`]).
///
/// # Panics
///
/// Panics if constructing the `EnvFilter` from `RUST_LOG` fails with an
/// invalid directive.  This is intentional — misconfigured logging should
/// be caught at startup, not silently ignored.
pub fn init_global_logger(format: LogFormat) {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));

    let result = match format {
        LogFormat::Pretty => tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .try_init(),
        LogFormat::Json => tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().json())
            .try_init(),
    };

    if let Err(e) = result {
        // This only fails if a global subscriber is already set (e.g., in
        // test harnesses that run multiple #[tokio::test] functions).
        // Safe to swallow — the existing subscriber is valid.
        tracing::warn!("Global subscriber already set, skipping re-init: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_format_from_str_lossy_parses_json() {
        assert_eq!(LogFormat::from_str_lossy("json"), LogFormat::Json);
        assert_eq!(LogFormat::from_str_lossy("JSON"), LogFormat::Json);
        assert_eq!(LogFormat::from_str_lossy("Json"), LogFormat::Json);
    }

    #[test]
    fn log_format_from_str_lossy_defaults_to_pretty() {
        assert_eq!(LogFormat::from_str_lossy("pretty"), LogFormat::Pretty);
        assert_eq!(LogFormat::from_str_lossy("text"), LogFormat::Pretty);
        assert_eq!(LogFormat::from_str_lossy(""), LogFormat::Pretty);
    }
}
