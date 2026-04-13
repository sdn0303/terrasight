//! Thin wrapper that delegates to [`terrasight_server::log`].
//!
//! Keeps `main.rs` imports stable while the underlying telemetry
//! implementation evolves independently in the `lib/server` crate.

use crate::config::Config;
use terrasight_server::log::LogFormat;

/// Default env filter when `RUST_LOG` is not set.
///
/// - Application crates at `info`
/// - SQLx internal noise at `warn`
/// - tower-http request tracing at `debug`
const DEFAULT_FILTER: &str = "\
    realestate_api=info,\
    terrasight_server=info,\
    realestate_geo_math=debug,\
    mlit_client=info,\
    sqlx=warn,\
    tower_http=debug,\
    hyper=warn\
";

/// Initialize structured logging based on application configuration.
///
/// Delegates to [`terrasight_server::log::init_global_logger`].
pub fn init(config: &Config) {
    let format = config
        .rust_log_format
        .as_deref()
        .map(LogFormat::from_str_lossy)
        .unwrap_or(LogFormat::Pretty);

    terrasight_server::log::init_global_logger(format, Some(DEFAULT_FILTER));
}
